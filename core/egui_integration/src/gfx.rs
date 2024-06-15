use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{ClippedPrimitive, ImageData, Mesh, Rect, TextureId, TexturesDelta};
use egui::epaint::{ImageDelta, Primitive, Vertex};
use wgpu::{BindGroup, BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress, ColorTargetState, CommandEncoder, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, IndexFormat, Origin3d, PipelineLayout, Queue, RenderPipeline, ShaderStages, Texture, TextureAspect, TextureFormat, TextureView, VertexBufferLayout, VertexStepMode};

use common::screen::ScreenSize;
use gfx::bind_group::{BindGroupBuilder, BindGroupLayoutBuilder, CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::bind_group::layout_entry::BindGroupLayoutEntryBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::growable_buffer::{GrowableBuffer, GrowableBufferBuilder};
use gfx::include_spirv_shader;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};

pub struct GuiGfx {
  index_buffer: GrowableBuffer,
  vertex_buffer: GrowableBuffer,
  uniform_buffer: GfxBuffer,
  static_bind_group: CombinedBindGroup,
  texture_bind_group_layout: BindGroupLayout,
  textures: HashMap<TextureId, (GfxTexture, BindGroup)>,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,

  free_textures: Vec<TextureId>,
}

impl GuiGfx {
  pub fn new(device: &Device, swap_chain_texture_format: TextureFormat) -> Self {
    let vertex_shader_module = device.create_shader_module(include_spirv_shader!("gui.vert"));
    let fragment_shader_module = device.create_shader_module(include_spirv_shader!("gui.frag"));

    let index_buffer = GrowableBufferBuilder::default()
      .label("GUI index buffer")
      .index_usage()
      .grow_multiplier(1.5)
      .build();
    let vertex_buffer = GrowableBufferBuilder::default()
      .label("GUI vertex buffer")
      .vertex_usage()
      .grow_multiplier(1.5)
      .build();

    // Bind group that does not change while rendering (static), containing the uniform buffer and texture sampler.
    let uniform_buffer = BufferBuilder::new()
      .uniform_usage()
      .label("GUI uniform buffer")
      .build_with_data(device, &[Uniform::default()]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::VERTEX);

    let sampler = SamplerBuilder::new()
      .mag_filter(FilterMode::Linear)
      .min_filter(FilterMode::Linear)
      .label("GUI texture sampler")
      .build(device);
    let sampler_binding = sampler.binding(1, ShaderStages::FRAGMENT);

    let static_bind_group = CombinedBindGroupBuilder::default()
      .layout_entries(&[uniform_binding.layout, sampler_binding.layout])
      .layout_label("GUI static bind group layout")
      .entries(&[uniform_binding.entry, sampler_binding.entry])
      .label("GUI static bind group")
      .build(device);

    // Bind group that does change while rendering, containing the current texture.
    let texture_layout = BindGroupLayoutEntryBuilder::default()
      .texture()
      .binding(0)
      .fragment_visibility()
      .build();
    let texture_bind_group_layout = BindGroupLayoutBuilder::default()
      .entries(&[texture_layout])
      .label("GUI texture bind group layout")
      .build(device);

    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::default()
      .layout_label("GUI pipeline layout")
      .bind_group_layouts(&[&static_bind_group.layout, &texture_bind_group_layout])
      .label("GUI render pipeline")
      .vertex_module(&vertex_shader_module)
      .vertex_buffer_layouts(&[VertexBufferLayout {
        // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L174-L180
        array_stride: 5 * 4,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32],
      }])
      .fragment_module(&fragment_shader_module)
      .fragment_targets(&[Some(ColorTargetState {
        blend: Some(BlendState {
          color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
          },
          alpha: BlendComponent {
            // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L201-L210
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
          },
        }),
        ..swap_chain_texture_format.into()
      })])
      .build(device);

    Self {
      index_buffer,
      vertex_buffer,
      uniform_buffer,
      static_bind_group,
      texture_bind_group_layout,
      textures: HashMap::default(),
      _pipeline_layout: pipeline_layout,
      render_pipeline,

      free_textures: Vec::default(),
    }
  }


  #[profiling::function]
  pub fn update_textures(&mut self, device: &Device, queue: &Queue, textures_delta: TexturesDelta) {
    for (texture_id, image_delta) in &textures_delta.set {
      self.update_texture(device, queue, *texture_id, image_delta);
    }
    self.free_textures = textures_delta.free;
  }

  #[profiling::function]
  fn update_texture(&mut self, device: &Device, queue: &Queue, id: TextureId, image_delta: &ImageDelta) {
    // From:
    // 1) https://github.com/hasenbanck/egui_wgpu_backend/blob/b2d3e7967351690c6425f37cd6d4ffb083a7e8e6/src/lib.rs#L379
    // 2) https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/renderer.rs#L500

    let label_base = match id {
      TextureId::Managed(m) => format!("egui managed texture {}", m),
      TextureId::User(u) => format!("egui user texture {}", u),
    };

    let width = image_delta.image.width() as u32;
    let height = image_delta.image.height() as u32;
    let size = Extent3d {
      width,
      height,
      depth_or_array_layers: 1,
    };
    let layout = ImageDataLayout {
      offset: 0,
      bytes_per_row: Some(4 * width),
      rows_per_image: None,
    };

    let pixels = match &image_delta.image {
      ImageData::Color(image) => Cow::Borrowed(&image.pixels),
      ImageData::Font(image) => Cow::Owned(image.srgba_pixels(None).collect()),
    };
    let data = bytemuck::cast_slice(pixels.as_slice());

    match self.textures.entry(id) {
      Entry::Occupied(mut o) => match image_delta.pos {
        Some([x, y]) => {
          let origin = Origin3d { x: x as u32, y: y as u32, z: 0 };
          write_texture(queue, &o.get().0.texture, origin, data, layout, size);
        }
        None => {
          let (texture, bind_group) = create_texture_and_bind_group(
            device,
            queue,
            &label_base,
            size,
            data,
            layout,
            &self.texture_bind_group_layout,
          );
          let (texture, _) = o.insert((texture, bind_group));
          texture.texture.destroy();
        }
      },
      Entry::Vacant(v) => {
        let (texture, bind_group) = create_texture_and_bind_group(
          device,
          queue,
          &label_base,
          size,
          data,
          layout,
          &self.texture_bind_group_layout,
        );
        v.insert((texture, bind_group));
      }
    }
  }

  #[profiling::function]
  pub fn render(
    &mut self,
    device: &Device,
    queue: &Queue,
    clipped_primitives: Vec<ClippedPrimitive>,
    screen_size: ScreenSize,
    surface_texture_view: &TextureView,
    encoder: &mut CommandEncoder,
  ) {
    // Ensure index and vertex buffers are big enough.
    let index_count: usize = clipped_primitives.iter()
      .map(|cp| if let Primitive::Mesh(Mesh { indices, .. }) = &cp.primitive { indices.len() } else { 0 })
      .sum();
    let index_buffer = self.index_buffer.ensure_minimum_size(device, (index_count * size_of::<u32>()) as BufferAddress);
    let vertex_count: usize = clipped_primitives.iter()
      .map(|cp| if let Primitive::Mesh(Mesh { vertices, .. }) = &cp.primitive { vertices.len() } else { 0 })
      .sum();
    let vertex_buffer = self.vertex_buffer.ensure_minimum_size(device, (vertex_count * size_of::<Vertex>()) as BufferAddress);

    // Write to buffers and create draw list.
    self.uniform_buffer.write_all_data(queue, &[Uniform::from_screen_size(screen_size)]);
    let mut index_offset = 0;
    let mut index_buffer_offset = 0;
    let mut vertex_offset = 0;
    let mut vertex_buffer_offset = 0;
    #[derive(Debug)]
    struct Draw {
      clip_rect: Rect,
      texture_id: TextureId,
      indices: Range<u32>,
      base_vertex: usize,
    }
    let mut draws = Vec::with_capacity(clipped_primitives.len());
    for ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
      if let Primitive::Mesh(Mesh { indices, vertices, texture_id, .. }) = primitive {
        index_buffer.write_data(queue, &indices, index_buffer_offset);
        vertex_buffer.write_data(queue, &vertices, vertex_buffer_offset);
        draws.push(Draw { clip_rect, texture_id, indices: index_offset..index_offset + indices.len() as u32, base_vertex: vertex_offset });
        index_offset += indices.len() as u32;
        index_buffer_offset += indices.len();
        vertex_offset += vertices.len();
        vertex_buffer_offset += vertices.len();
      }
    }

    // Render
    {
      let mut bound_texture_id = None;
      let mut render_pass = RenderPassBuilder::new()
        .with_label("GUI render pass")
        .begin_render_pass_for_swap_chain_with_load(encoder, surface_texture_view);
      render_pass.push_debug_group("Draw GUI");
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.static_bind_group.entry, &[]);
      render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
      let scale_factor: f64 = screen_size.scale.into();
      let scale_factor = scale_factor as f32;
      let physical_width = screen_size.physical.width as u32;
      let physical_height = screen_size.physical.height as u32;
      for Draw { clip_rect, texture_id, indices, base_vertex } in draws {
        // Taken from:
        // - https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L272-L305
        // - https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/renderer.rs#L983

        // Transform clip rect to physical pixels.
        let clip_min_x = scale_factor * clip_rect.min.x;
        let clip_min_y = scale_factor * clip_rect.min.y;
        let clip_max_x = scale_factor * clip_rect.max.x;
        let clip_max_y = scale_factor * clip_rect.max.y;

        // Round to integer.
        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;

        // Clamp to physical pixels.
        let clip_min_x = clip_min_x.clamp(0, physical_width);
        let clip_min_y = clip_min_y.clamp(0, physical_height);
        let clip_max_x = clip_max_x.clamp(clip_min_x, physical_width);
        let clip_max_y = clip_max_y.clamp(clip_min_y, physical_height);

        let x = clip_min_x;
        let y = clip_min_y;
        let width = clip_max_x - clip_min_x;
        let height = clip_max_y - clip_min_y;

        // Skip rendering with zero-sized clip areas.
        if width == 0 || height == 0 {
          continue;
        }

        render_pass.set_scissor_rect(x, y, width, height);

        match bound_texture_id {
          Some(bound_id) if texture_id == bound_id => {
            // Do nothing, already bound correct group.
          }
          ref mut b => {
            let bind_group = &self.textures
              .get(&texture_id)
              .unwrap_or_else(|| panic!("Cannot get bind group for {:?}; texture with that ID does not exist", texture_id)).1;
            render_pass.set_bind_group(1, bind_group, &[]);
            *b = Some(texture_id);
          }
        }

        // Use `set_vertex_buffer` with offset and `draw_indexed` with `base_vertex` = 0 for WebGL2 support.
        render_pass.set_vertex_buffer(0, vertex_buffer.slice_data::<Vertex>(base_vertex..));
        render_pass.draw_indexed(indices, 0, 0..1);
      }
      render_pass.pop_debug_group();
    }

    self.free_textures();
  }

  #[profiling::function]
  pub fn free_textures(&mut self) {
    for texture_id in self.free_textures.drain(..) {
      let (texture, _bind_group) = self.textures.
        remove(&texture_id)
        .unwrap_or_else(|| panic!("Cannot free texture for {:?}; texture with that ID does not exist", texture_id));
      texture.texture.destroy();
    }
  }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Pod, Zeroable)]
struct Uniform {
  // Note: array of size 4 due to alignment requirements for uniform buffers.
  screen_size: [f32; 4],
}
impl Uniform {
  #[inline]
  fn from_screen_size(screen_size: ScreenSize) -> Self {
    Self { screen_size: [screen_size.logical.width as f32, screen_size.logical.height as f32, 0.0, 0.0] }
  }
}

#[profiling::function]
fn create_texture_and_bind_group(
  device: &Device,
  queue: &Queue,
  label_base: &str,
  size: Extent3d,
  data: &[u8],
  layout: ImageDataLayout,
  texture_bind_group_layout: &BindGroupLayout,
) -> (GfxTexture, BindGroup) {
  let texture = TextureBuilder::new()
    .with_texture_label(label_base)
    .with_size(size)
    .with_rgba8_unorm_srgb_format()
    .with_sampled_usage()
    .with_texture_view_label(&format!("{} view", label_base))
    .build(device);
  write_texture(queue, &texture.texture, Origin3d::ZERO, data, layout, size);
  let bind_group = BindGroupBuilder::default()
    .entries(&[texture.entry(0)])
    .label(&format!("{} bind group", label_base))
    .build(device, texture_bind_group_layout);
  (texture, bind_group)
}

#[profiling::function]
fn write_texture(
  queue: &Queue,
  texture: &Texture,
  origin: Origin3d,
  data: &[u8],
  layout: ImageDataLayout,
  size: Extent3d,
) {
  let copy = ImageCopyTexture { texture, mip_level: 0, origin, aspect: TextureAspect::All };
  queue.write_texture(copy, data, layout, size);
}
