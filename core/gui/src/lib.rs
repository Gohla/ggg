use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;
use std::num::NonZeroU32;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{ClippedPrimitive, Context, Event, ImageData, Key, PointerButton, Pos2, RawInput as EguiRawInput, Rect, TextureId, TexturesDelta, Vec2};
use egui::epaint::{Mesh, Primitive, Vertex};
use wgpu::{BindGroup, BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress, ColorTargetState, CommandEncoder, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, IndexFormat, Origin3d, PipelineLayout, Queue, RenderPipeline, ShaderStages, TextureAspect, TextureFormat, TextureView, VertexBufferLayout, VertexStepMode};

use common::input::{KeyboardButton, KeyboardModifier, MouseButton, RawInput};
use common::screen::ScreenSize;
use gfx::bind_group::{BindGroupBuilder, BindGroupLayoutBuilder, BindGroupLayoutEntryBuilder, CombinedBindGroupLayoutBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::prelude::*;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};

pub struct Gui {
  pub context: Context,
  input: EguiRawInput,

  index_buffer: Option<GfxBuffer>,
  vertex_buffer: Option<GfxBuffer>,
  uniform_buffer: GfxBuffer,
  _static_bind_group_layout: BindGroupLayout,
  static_bind_group: BindGroup,
  texture_bind_group_layout: BindGroupLayout,
  textures: HashMap<TextureId, (GfxTexture, BindGroup)>,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
}

// Creation

impl Gui {
  pub fn new(
    device: &Device,
    swap_chain_texture_format: TextureFormat,
    memory: Option<egui::Memory>,
  ) -> Self {
    let context = Context::default();
    if let Some(memory) = memory {
      *context.memory() = memory;
    }
    
    let vertex_shader_module = device.create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader/gui.vert.spv")));
    let fragment_shader_module = device.create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader/gui.frag.spv")));
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("GUI uniform buffer")
      .build_with_data(device, &[Uniform::default()]);
    let (uniform_buffer_bind_layout_entry, uniform_buffer_bind_entry) =
      uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX);
    let sampler = SamplerBuilder::new()
      .with_mag_filter(FilterMode::Linear)
      .with_min_filter(FilterMode::Linear)
      .with_label("GUI texture sampler")
      .build(device);
    let (sampler_bind_layout_entry, sampler_bind_entry) = sampler.create_bind_group_entries(1, ShaderStages::FRAGMENT);
    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_buffer_bind_layout_entry, sampler_bind_layout_entry])
      .with_layout_label("GUI static bind group layout")
      .with_entries(&[uniform_buffer_bind_entry, sampler_bind_entry])
      .with_label("GUI static bind group")
      .build(device);
    let texture_bind_group_layout = BindGroupLayoutBuilder::new()
      .with_entries(&[BindGroupLayoutEntryBuilder::new_default_float_2d_texture()
        .with_binding(0)
        .with_fragment_shader_visibility()
        .build()
      ])
      .with_label("GUI texture bind group layout")
      .build(device);
    let textures = HashMap::new();
    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&static_bind_group_layout, &texture_bind_group_layout])
      .with_vertex_buffer_layouts(&[VertexBufferLayout { // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L174-L180
        array_stride: 5 * 4,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32],
      }])
      .with_fragment_state(&fragment_shader_module, "main", &[Some(ColorTargetState {
        format: swap_chain_texture_format,
        blend: Some(BlendState {
          alpha: BlendComponent { // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L201-L210
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
          },
          color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
          },
        }),
        write_mask: Default::default(),
      })])
      .with_layout_label("GUI pipeline layout")
      .with_label("GUI render pipeline")
      .build(device);
    
    Self {
      context,
      input: EguiRawInput::default(),
      index_buffer: None,
      vertex_buffer: None,
      uniform_buffer,
      _static_bind_group_layout: static_bind_group_layout,
      static_bind_group,
      texture_bind_group_layout,
      textures,
      _pipeline_layout: pipeline_layout,
      render_pipeline,
    }
  }
}

// Input processing.

impl Gui {
  #[profiling::function]
  pub fn process_input(&mut self, input: &RawInput, process_keyboard_input: bool, process_mouse_input: bool) {
    if process_keyboard_input {
      // Keyboard modifiers
      self.input.modifiers.shift = input.is_keyboard_modifier_down(KeyboardModifier::Shift);
      let is_control_down = input.is_keyboard_modifier_down(KeyboardModifier::Control);
      self.input.modifiers.ctrl = is_control_down;
      #[cfg(not(target_os = "macos"))] {
        self.input.modifiers.command = is_control_down;
      }
      self.input.modifiers.alt = input.is_keyboard_modifier_down(KeyboardModifier::Alternate);
      #[cfg(target_os = "macos")] {
        let is_meta_down = input.is_keyboard_modifier_down(KeyboardModifier::Meta);
        self.input.modifiers.mac_cmd = is_meta_down;
        self.input.modifiers.command = is_meta_down;
      }
    }
    let modifiers = self.input.modifiers;

    if process_mouse_input {
      // Mouse wheel delta // TODO: properly handle line to pixel conversion?
      if !input.mouse_wheel_pixel_delta.is_zero() || !input.mouse_wheel_line_delta.is_zero() {
        self.input.events.push(Event::Scroll(Vec2::new(
          (input.mouse_wheel_pixel_delta.logical.x as f64 + input.mouse_wheel_line_delta.horizontal * 24.0) as f32,
          (input.mouse_wheel_pixel_delta.logical.y as f64 + input.mouse_wheel_line_delta.vertical * 24.0) as f32,
        )));
      }

      // Mouse movement
      let mouse_position = Pos2::new(input.mouse_position.logical.x as f32, input.mouse_position.logical.y as f32);
      if !input.mouse_position_delta.is_zero() {
        self.input.events.push(Event::PointerMoved(mouse_position))
      }

      // Mouse buttons
      fn convert_mouse_button(mouse_button: MouseButton) -> Option<PointerButton> {
        match mouse_button {
          MouseButton::Left => Some(PointerButton::Primary),
          MouseButton::Right => Some(PointerButton::Secondary),
          MouseButton::Middle => Some(PointerButton::Middle),
          MouseButton::Other(_) => None
        }
      }
      for button in &input.mouse_buttons_pressed {
        if let Some(button) = convert_mouse_button(*button) {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: true, modifiers })
        }
      }
      for button in &input.mouse_buttons_released {
        if let Some(button) = convert_mouse_button(*button) {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: false, modifiers })
        }
      }
    }

    if process_keyboard_input {
      // Keyboard buttons
      // Taken from: https://github.com/hasenbanck/egui_winit_platform/blob/400ebfc2f3e9a564a701406e724096556bb2f8c4/src/lib.rs#L262-L292
      fn convert_keyboard_button(button: KeyboardButton) -> Option<Key> {
        use KeyboardButton::*;
        Some(match button {
          Escape => Key::Escape,
          Insert => Key::Insert,
          Home => Key::Home,
          Delete => Key::Delete,
          End => Key::End,
          PageDown => Key::PageDown,
          PageUp => Key::PageUp,
          Left => Key::ArrowLeft,
          Up => Key::ArrowUp,
          Right => Key::ArrowRight,
          Down => Key::ArrowDown,
          Back => Key::Backspace,
          Return => Key::Enter,
          Tab => Key::Tab,
          Space => Key::Space,

          A => Key::A,
          K => Key::K,
          U => Key::U,
          W => Key::W,
          Z => Key::Z,

          _ => { return None; }
        })
      }
      for button in &input.keyboard_buttons_pressed {
        if let Some(key) = convert_keyboard_button(*button) {
          self.input.events.push(Event::Key { key, pressed: true, modifiers })
        }
      }
      for button in &input.keyboard_buttons_released {
        if let Some(key) = convert_keyboard_button(*button) {
          self.input.events.push(Event::Key { key, pressed: false, modifiers })
        }
      }

      // Characters
      for character in &input.characters_pressed {
        let character = *character;
        // Taken from: https://github.com/hasenbanck/egui_winit_platform/blob/400ebfc2f3e9a564a701406e724096556bb2f8c4/src/lib.rs#L312-L320
        let is_in_private_use_area = '\u{e000}' <= character && character <= '\u{f8ff}'
          || '\u{f0000}' <= character && character <= '\u{ffffd}'
          || '\u{100000}' <= character && character <= '\u{10fffd}';
        if !is_in_private_use_area && !character.is_ascii_control() {
          self.input.events.push(Event::Text(character.to_string()))
        }
      }
    }
  }

  pub fn is_capturing_keyboard(&self) -> bool { self.context.wants_keyboard_input() }

  pub fn is_capturing_mouse(&self) -> bool { self.context.wants_pointer_input() }
}

// Begin GUI frame, returning the context to start building the GUI.

impl Gui {
  #[profiling::function]
  pub fn begin_frame(
    &mut self,
    screen_size: ScreenSize,
    elapsed_seconds: f64,
    delta_seconds: f64,
  ) -> Context {
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(screen_size.physical.width as f32, screen_size.physical.height as f32));
    self.input.screen_rect = Some(screen_rect);
    let pixels_per_point: f64 = screen_size.scale.into();
    self.input.pixels_per_point = Some(pixels_per_point as f32);

    self.input.time = Some(elapsed_seconds);
    self.input.predicted_dt = delta_seconds as f32;

    let input = std::mem::take(&mut self.input);
    self.context.begin_frame(input);
    self.context.clone()
  }
}

// Rendering the built GUI.

impl Gui {
  #[profiling::function]
  pub fn render(
    &mut self,
    screen_size: ScreenSize,
    device: &Device,
    queue: &Queue,
    encoder: &mut CommandEncoder,
    surface_texture_view: &TextureView,
  ) {
    // End the frame to get output.
    let full_output = self.context.end_frame();

    // Update textures
    self.set_textures(device, queue, &full_output.textures_delta);

    // Get primitives to render.
    let clipped_primitives: Vec<ClippedPrimitive> = self.context.tessellate(full_output.shapes);

    // (Re-)Create index and vertex buffers if they do not exist yet or are not large enough.
    let index_buffer = {
      let count: usize = clipped_primitives.iter()
        .map(|cp| if let Primitive::Mesh(Mesh { indices, .. }) = &cp.primitive { indices.len() } else { 0 })
        .sum();
      let size = (count * size_of::<u32>()) as BufferAddress;
      let mut buffer = self.index_buffer.take().unwrap_or_else(|| create_index_buffer(size, device));
      if buffer.size < size {
        buffer.destroy();
        // Double size to prevent rapid buffer recreation. // TODO: will never shrink, is that ok?
        buffer = create_index_buffer(buffer.size.saturating_mul(2).max(size), device);
      }
      buffer
    };
    let vertex_buffer = {
      let count: usize = clipped_primitives.iter()
        .map(|cp| if let Primitive::Mesh(Mesh { vertices, .. }) = &cp.primitive { vertices.len() } else { 0 })
        .sum();
      let size = (count * size_of::<Vertex>()) as BufferAddress;
      let mut buffer = self.vertex_buffer.take().unwrap_or_else(|| create_vertex_buffer(size, device));
      if buffer.size < size {
        buffer.destroy();
        // Double size to prevent rapid buffer recreation. // TODO: will never shrink, is that ok?
        buffer = create_vertex_buffer(buffer.size.saturating_mul(2).max(size), device);
      }
      buffer
    };

    // Write to buffers and create draw list.
    self.uniform_buffer.write_whole_data(queue, &[Uniform::from_screen_size(screen_size)]);
    let mut index_offset = 0;
    let mut index_buffer_offset = 0;
    let mut vertex_offset = 0;
    let mut vertex_buffer_offset = 0;
    #[derive(Debug)]
    struct Draw {
      clip_rect: Rect,
      texture_id: TextureId,
      indices: Range<u32>,
      base_vertex: u64,
    }
    let mut draws = Vec::with_capacity(clipped_primitives.len());
    for ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
      if let Primitive::Mesh(Mesh { indices, vertices, texture_id, .. }) = primitive {
        index_buffer.write_data(queue, index_buffer_offset, &indices);
        vertex_buffer.write_bytes(queue, vertex_buffer_offset, bytemuck::cast_slice(&vertices));
        draws.push(Draw { clip_rect, texture_id, indices: index_offset..index_offset + indices.len() as u32, base_vertex: vertex_offset });
        index_offset += indices.len() as u32;
        index_buffer_offset += (indices.len() * size_of::<u32>()) as BufferAddress;
        vertex_offset += vertices.len() as u64;
        vertex_buffer_offset += (vertices.len() * size_of::<Vertex>()) as BufferAddress;
      }
    }

    // Render
    {
      let mut render_pass = RenderPassBuilder::new()
        .with_label("GUI render pass")
        .begin_render_pass_for_swap_chain_with_load(encoder, surface_texture_view);
      render_pass.push_debug_group("Draw GUI");
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.static_bind_group, &[]);
      let mut bound_texture_id = TextureId::default();
      render_pass.set_bind_group(1, self.get_texture_bind_group(bound_texture_id), &[]);
      render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
      let scale_factor: f64 = screen_size.scale.into();
      let scale_factor = scale_factor as f32;
      let physical_width = screen_size.physical.width as u32;
      let physical_height = screen_size.physical.height as u32;
      for Draw { clip_rect, texture_id, indices, base_vertex } in draws {
        // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L272-L305
        // Transform clip rect to physical pixels.
        let clip_min_x = scale_factor * clip_rect.min.x;
        let clip_min_y = scale_factor * clip_rect.min.y;
        let clip_max_x = scale_factor * clip_rect.max.x;
        let clip_max_y = scale_factor * clip_rect.max.y;

        // Make sure clip rect can fit within an `u32`.
        let clip_min_x = clip_min_x.clamp(0.0, physical_width as f32);
        let clip_min_y = clip_min_y.clamp(0.0, physical_height as f32);
        let clip_max_x = clip_max_x.clamp(clip_min_x, physical_width as f32);
        let clip_max_y = clip_max_y.clamp(clip_min_y, physical_height as f32);

        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;

        let width = (clip_max_x - clip_min_x).max(1);
        let height = (clip_max_y - clip_min_y).max(1);

        // Clip scissor rectangle to target size
        let x = clip_min_x.min(physical_width);
        let y = clip_min_y.min(physical_height);
        let width = width.min(physical_width - x);
        let height = height.min(physical_height - y);

        // Skip rendering with zero-sized clip areas.
        if width == 0 || height == 0 {
          continue;
        }

        render_pass.set_scissor_rect(x, y, width, height);
        if texture_id != bound_texture_id {
          bound_texture_id = texture_id;
          render_pass.set_bind_group(1, self.get_texture_bind_group(bound_texture_id), &[]);
        }
        // Use `set_vertex_buffer` with offset and `draw_indexed` with `base_vertex` = 0 for WebGL2 support.
        render_pass.set_vertex_buffer(0, vertex_buffer.offset::<Vertex>(base_vertex));
        render_pass.draw_indexed(indices, 0, 0..1);
      }
      render_pass.pop_debug_group();
    }

    // Store index and vertex buffers (outside of render pass because it held a reference to these buffers).
    self.index_buffer = Some(index_buffer);
    self.vertex_buffer = Some(vertex_buffer);

    // Free unused textures.
    self.free_textures(&full_output.textures_delta);
  }

  fn set_textures(&mut self, device: &Device, queue: &Queue, textures_delta: &TexturesDelta) {
    // From: https://github.com/hasenbanck/egui_wgpu_backend/blob/b2d3e7967351690c6425f37cd6d4ffb083a7e8e6/src/lib.rs#L379
    for (texture_id, image_delta) in textures_delta.set.iter() {
      let image_size = image_delta.image.size();
      let origin = match image_delta.pos {
        Some([x, y]) => Origin3d {
          x: x as u32,
          y: y as u32,
          z: 0,
        },
        None => Origin3d::ZERO,
      };
      let alpha_srgb_pixels: Option<Vec<_>> = match &image_delta.image {
        ImageData::Color(_) => None,
        ImageData::Font(f) => Some(f.srgba_pixels(1.0).collect()),
      };
      let image_data: &[u8] = match &image_delta.image {
        ImageData::Color(c) => bytemuck::cast_slice(c.pixels.as_slice()),
        ImageData::Font(_) => {
          // unwrap should never fail as alpha_srgb_pixels will have been set to `Some` above.
          bytemuck::cast_slice(alpha_srgb_pixels.as_ref().unwrap().as_slice())
        }
      };
      let image_size = Extent3d {
        width: image_size[0] as u32,
        height: image_size[1] as u32,
        depth_or_array_layers: 1,
      };
      let image_data_layout = ImageDataLayout {
        offset: 0,
        bytes_per_row: NonZeroU32::new(4 * image_size.width),
        rows_per_image: None,
      };
      let label_base = match texture_id {
        TextureId::Managed(m) => format!("EGUI managed texture {}", m),
        TextureId::User(u) => format!("EGUI user texture {}", u),
      };
      match self.textures.entry(*texture_id) {
        Entry::Occupied(mut o) => match image_delta.pos {
          None => {
            let (texture, bind_group) = create_texture_and_bind_group(
              device,
              queue,
              &label_base,
              origin,
              image_data,
              image_data_layout,
              image_size,
              &self.texture_bind_group_layout,
            );
            let (texture, _) = o.insert((texture, bind_group));
            texture.texture.destroy();
          }
          Some(_) => {
            queue.write_texture(
              ImageCopyTexture {
                texture: &o.get().0.texture,
                mip_level: 0,
                origin,
                aspect: TextureAspect::All,
              },
              image_data,
              image_data_layout,
              image_size,
            );
          }
        },
        Entry::Vacant(v) => {
          let (texture, bind_group) = create_texture_and_bind_group(
            device,
            queue,
            &label_base,
            origin,
            image_data,
            image_data_layout,
            image_size,
            &self.texture_bind_group_layout,
          );
          v.insert((texture, bind_group));
        }
      }
    }
  }

  fn get_texture_bind_group(&self, texture_id: TextureId) -> &BindGroup {
    &self.textures
      .get(&texture_id)
      .unwrap_or_else(|| panic!("Cannot get bind group for {:?}; texture with that ID does not exist", texture_id)).1
  }

  fn free_textures(&mut self, textures_delta: &TexturesDelta) {
    for texture_id in textures_delta.free.iter() {
      let (texture, _bind_group) = self.textures.
        remove(&texture_id)
        .unwrap_or_else(|| panic!("Cannot free texture for {:?}; texture with that ID does not exist", texture_id));
      texture.texture.destroy();
    }
  }
}

// Utilities

#[inline]
fn create_index_buffer(size: BufferAddress, device: &Device) -> GfxBuffer {
  BufferBuilder::new()
    .with_index_usage()
    .with_size(size)
    .with_label("GUI index buffer")
    .build(device)
}

#[inline]
fn create_vertex_buffer(size: BufferAddress, device: &Device) -> GfxBuffer {
  BufferBuilder::new()
    .with_vertex_usage()
    .with_size(size)
    .with_label("GUI vertex buffer")
    .build(device)
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Pod, Zeroable)]
struct Uniform {
  screen_size: [f32; 4],
}

impl Uniform {
  #[inline]
  pub fn from_screen_size(screen_size: ScreenSize) -> Self {
    Self { screen_size: [screen_size.logical.width as f32, screen_size.logical.height as f32, 0.0, 0.0] }
  }
}

fn create_texture_and_bind_group(
  device: &Device,
  queue: &Queue,
  label_base: &str,
  origin: Origin3d,
  image_data: &[u8],
  image_data_layout: ImageDataLayout,
  image_size: Extent3d,
  texture_bind_group_layout: &BindGroupLayout,
) -> (GfxTexture, BindGroup) {
  let texture = TextureBuilder::new()
    .with_size(image_size)
    .with_rgba8_unorm_srgb_format()
    .with_sampled_usage()
    .with_texture_label(label_base)
    .with_texture_view_label(&format!("{} view", label_base))
    .build(device);
  queue.write_texture(
    ImageCopyTexture {
      texture: &texture.texture,
      mip_level: 0,
      origin,
      aspect: TextureAspect::All,
    },
    image_data,
    image_data_layout,
    image_size,
  );
  let bind_group = BindGroupBuilder::new(texture_bind_group_layout)
    .with_entries(&[texture.create_bind_group_entry(0)])
    .with_label(&format!("{} bind group", label_base))
    .build(device);
  (texture, bind_group)
}
