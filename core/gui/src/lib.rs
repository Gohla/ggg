use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{ClippedPrimitive, Context, CursorIcon, Event, ImageData, MouseWheelUnit, PlatformOutput, Pos2, RawInput as EguiRawInput, Rect, TextureId, TexturesDelta};
use egui::epaint::{ImageDelta, Mesh, Primitive, Vertex};
use wgpu::{BindGroup, BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress, ColorTargetState, CommandEncoder, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, IndexFormat, Origin3d, PipelineLayout, Queue, RenderPipeline, ShaderStages, Texture, TextureAspect, TextureFormat, TextureView, VertexBufferLayout, VertexStepMode};

use common::input::{Key, KeyboardModifier, RawInput};
use common::screen::ScreenSize;
use gfx::bind_group::{BindGroupBuilder, BindGroupLayoutBuilder, BindGroupLayoutEntryBuilder, CombinedBindGroupLayoutBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::prelude::*;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use os::clipboard::{get_clipboard, TextClipboard};
use os::open_url::open_url;
use os::window::Window;

pub struct Gui {
  pub context: Context,
  clipboard: Box<dyn TextClipboard + Send + 'static>,

  input: EguiRawInput,

  cursor_icon: Option<CursorIcon>,
  cursor_in_window: bool,

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
      context.memory_mut(|m| *m = memory);
    }

    let vertex_shader_module = device.create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader/gui.vert.spv")));
    let fragment_shader_module = device.create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader/gui.frag.spv")));

    // Bind group that does not change while rendering (static), containing the uniform buffer and texture sampler.
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("GUI uniform buffer")
      .create_with_data(device, &[Uniform::default()]);
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

    // Bind group that does change while rendering, containing the current texture.
    let texture_bind_group_layout = BindGroupLayoutBuilder::new()
      .with_entries(&[BindGroupLayoutEntryBuilder::new_default_float_2d_texture()
        .with_binding(0)
        .with_fragment_shader_visibility()
        .build()
      ])
      .with_label("GUI texture bind group layout")
      .build(device);

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
          color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
          },
          alpha: BlendComponent { // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L201-L210
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
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
      clipboard: get_clipboard(),

      input: EguiRawInput::default(),

      cursor_icon: None,
      cursor_in_window: false,

      index_buffer: None,
      vertex_buffer: None,
      uniform_buffer,
      _static_bind_group_layout: static_bind_group_layout,
      static_bind_group,
      texture_bind_group_layout,
      textures: HashMap::default(),
      _pipeline_layout: pipeline_layout,
      render_pipeline,
    }
  }


  //
  // Input processing.
  //

  #[profiling::function]
  pub fn process_input(&mut self, input: &RawInput, process_keyboard_input: bool, process_mouse_input: bool) {
    if process_keyboard_input {
      // Keyboard modifiers
      self.input.modifiers.alt = input.is_keyboard_modifier_down(KeyboardModifier::Alternate);
      let is_control_down = input.is_keyboard_modifier_down(KeyboardModifier::Control);
      self.input.modifiers.ctrl = is_control_down;
      self.input.modifiers.shift = input.is_keyboard_modifier_down(KeyboardModifier::Shift);
      let is_super_down = input.is_keyboard_modifier_down(KeyboardModifier::Super);
      self.input.modifiers.mac_cmd = cfg!(target_os = "macos") && is_super_down;
      self.input.modifiers.command = if cfg!(target_os = "macos") { is_super_down } else { is_control_down };
    }
    let modifiers = self.input.modifiers;

    if process_mouse_input {
      // Mouse wheel delta
      if !input.mouse_wheel_pixel_delta.is_zero() {
        let delta = input.mouse_wheel_pixel_delta.logical.into();
        self.input.events.push(Event::MouseWheel { unit: MouseWheelUnit::Point, delta, modifiers });
      }
      if !input.mouse_wheel_line_delta.is_zero() {
        let delta = input.mouse_wheel_line_delta.into();
        self.input.events.push(Event::MouseWheel { unit: MouseWheelUnit::Line, delta, modifiers });
      }

      // Mouse movement
      let mouse_position: Pos2 = input.mouse_position.logical.into();
      if !input.mouse_position_delta.is_zero() {
        self.cursor_in_window = true;
        self.input.events.push(Event::PointerMoved(mouse_position))
      }

      // Mouse buttons
      for button in input.mouse_buttons_pressed() {
        if let Some(button) = button.into() {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: true, modifiers })
        }
      }
      for button in input.mouse_buttons_released() {
        if let Some(button) = button.into() {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: false, modifiers })
        }
      }
    }

    if process_keyboard_input {
      fn is_cut_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Cut
          || (modifiers.command && keycode == egui::Key::X)
          || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Delete)
      }
      fn is_copy_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Copy
          || (modifiers.command && keycode == egui::Key::C)
          || (cfg!(target_os = "windows") && modifiers.ctrl && keycode == egui::Key::Insert)
      }
      fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Paste
          || (modifiers.command && keycode == egui::Key::V)
          || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Insert)
      }
      /// Ignore special keys (backspace, delete, F1, …) that winit sends as characters. Also ignore '\r', '\n', '\t'
      /// since newlines are handled by the `Key::Enter` event.
      ///
      /// From: https://github.com/emilk/egui/blob/9f12432bcf8f8275f154cbbb8aabdb8958be9026/crates/egui-winit/src/lib.rs#L991-L1001
      fn is_printable_char(chr: char) -> bool {
        let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
          || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
          || '\u{100000}' <= chr && chr <= '\u{10fffd}';
        !is_in_private_use_area && !chr.is_ascii_control()
      }

      // Keyboard keys
      for Key { keyboard, semantic, text } in input.keys_pressed() {
        let physical_key: Option<egui::Key> = keyboard.and_then(|k| k.into());
        let logical_key: Option<egui::Key> = semantic.and_then(|s| s.into());
        let handle_text = if let Some(key) = logical_key.or(physical_key) {
          if is_cut_command(modifiers, key) {
            self.input.events.push(Event::Cut);
            false
          } else if is_copy_command(modifiers, key) {
            self.input.events.push(Event::Copy);
            false
          } else if is_paste_command(modifiers, key) {
            if let Some(contents) = self.clipboard.get() {
              let contents = contents.replace("\r\n", "\n");
              if !contents.is_empty() {
                self.input.events.push(Event::Paste(contents));
              }
            }
            false
          } else {
            self.input.events.push(Event::Key { key, physical_key, pressed: true, repeat: false, modifiers });
            true
          }
        } else {
          true
        };

        // On some platforms we get here when the user presses Cmd-C (copy), ctrl-W, etc. We need to ignore these
        // characters that are side effects of commands.
        let is_cmd = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;
        if handle_text && !is_cmd {
          if let Some(text) = text {
            if !text.is_empty() && text.chars().all(is_printable_char) {
              self.input.events.push(Event::Text(text.to_string()))
            }
          }
        }
      }
      for Key { keyboard, semantic, .. } in input.keys_released() {
        let physical_key: Option<egui::Key> = keyboard.and_then(|k| k.into());
        let logical_key: Option<egui::Key> = semantic.and_then(|s| s.into());
        if let Some(key) = logical_key.or(physical_key) {
          self.input.events.push(Event::Key { key, physical_key, pressed: false, repeat: false, modifiers });
        }
        // Note: not handling text as egui doesn't need it for released keys.
      }
    }
  }
  pub fn is_capturing_keyboard(&self) -> bool { self.context.wants_keyboard_input() }
  pub fn is_capturing_mouse(&self) -> bool { self.context.wants_pointer_input() }

  pub fn update_window_cursor(&mut self, cursor_in_window: bool) {
    self.cursor_in_window = cursor_in_window;
    if !cursor_in_window {
      self.input.events.push(Event::PointerGone);
    }
  }

  pub fn update_window_focus(&mut self, focus: bool) {
    self.input.focused = focus;
    self.input.events.push(Event::WindowFocused(focus));
  }


  fn handle_platform_output(&mut self, window: &Window, platform_output: PlatformOutput) {
    self.set_cursor_icon(window, platform_output.cursor_icon);

    if let Some(url) = platform_output.open_url {
      open_url(&url.url, url.new_tab);
    }

    if !platform_output.copied_text.is_empty() {
      self.clipboard.set(&platform_output.copied_text)
    }
  }

  fn set_cursor_icon(&mut self, window: &Window, cursor_icon: CursorIcon) {
    if self.cursor_icon == Some(cursor_icon) {
      return;
    }

    if self.cursor_in_window {
      self.cursor_icon = Some(cursor_icon);
      window.set_option_cursor(cursor_icon.into())
    } else {
      self.cursor_icon = None;
    }
  }


  //
  // Begin GUI frame, returning the context to start building the GUI.
  //

  #[profiling::function]
  pub fn begin_frame(
    &mut self,
    screen_size: ScreenSize,
    elapsed_seconds: f64,
    delta_seconds: f64,
  ) -> Context {
    let screen_rect = Rect::from_min_size(Pos2::ZERO, screen_size.physical.into());
    self.input.screen_rect = Some(screen_rect);

    let native_pixels_per_point: f64 = screen_size.scale.into();
    if let Some(viewport) = self.input.viewports.get_mut(&self.input.viewport_id) {
      viewport.native_pixels_per_point = Some(native_pixels_per_point as f32);
    }

    self.input.time = Some(elapsed_seconds);
    self.input.predicted_dt = delta_seconds as f32;

    let input = std::mem::take(&mut self.input);
    self.context.begin_frame(input);
    self.context.clone()
  }


  //
  // Rendering the built GUI.
  //

  #[profiling::function]
  pub fn render(
    &mut self,
    window: &Window,
    screen_size: ScreenSize,
    device: &Device,
    queue: &Queue,
    encoder: &mut CommandEncoder,
    surface_texture_view: &TextureView,
  ) {
    // End the frame to get output.
    let full_output = self.context.end_frame();
    self.handle_platform_output(window, full_output.platform_output);

    // Update textures
    self.set_textures(device, queue, &full_output.textures_delta);

    // Get primitives to render.
    let clipped_primitives: Vec<ClippedPrimitive> = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

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
    self.uniform_buffer.enqueue_write_all_data(queue, &[Uniform::from_screen_size(screen_size)]);
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
        index_buffer.enqueue_write_data(queue, &indices, index_buffer_offset);
        vertex_buffer.enqueue_write_data(queue, &vertices, vertex_buffer_offset);
        draws.push(Draw { clip_rect, texture_id, indices: index_offset..index_offset + indices.len() as u32, base_vertex: vertex_offset });
        index_offset += indices.len() as u32;
        index_buffer_offset += indices.len();
        vertex_offset += vertices.len() as u64;
        vertex_buffer_offset += vertices.len();
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
        if texture_id != bound_texture_id {
          bound_texture_id = texture_id;
          render_pass.set_bind_group(1, self.get_texture_bind_group(bound_texture_id), &[]);
        }
        // Use `set_vertex_buffer` with offset and `draw_indexed` with `base_vertex` = 0 for WebGL2 support.
        render_pass.set_vertex_buffer(0, vertex_buffer.slice_to_end::<Vertex>(base_vertex));
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
    for (texture_id, image_delta) in &textures_delta.set {
      self.set_texture(device, queue, *texture_id, image_delta);
    }
  }

  // From:
  // 1) https://github.com/hasenbanck/egui_wgpu_backend/blob/b2d3e7967351690c6425f37cd6d4ffb083a7e8e6/src/lib.rs#L379
  // 2) https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/renderer.rs#L500
  fn set_texture(&mut self, device: &Device, queue: &Queue, id: TextureId, image_delta: &ImageDelta) {
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
    .create(device)
}

#[inline]
fn create_vertex_buffer(size: BufferAddress, device: &Device) -> GfxBuffer {
  BufferBuilder::new()
    .with_vertex_usage()
    .with_size(size)
    .with_label("GUI vertex buffer")
    .create(device)
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Pod, Zeroable)]
struct Uniform {
  // Note: array of size 4 due to alignment requirements for uniform buffers.
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
  let bind_group = BindGroupBuilder::new(texture_bind_group_layout)
    .with_entries(&[texture.create_bind_group_entry(0)])
    .with_label(&format!("{} bind group", label_base))
    .build(device);
  (texture, bind_group)
}

fn write_texture(
  queue: &Queue,
  texture: &Texture,
  origin: Origin3d,
  data: &[u8],
  layout: ImageDataLayout,
  size: Extent3d,
) {
  queue.write_texture(
    ImageCopyTexture {
      texture,
      mip_level: 0,
      origin,
      aspect: TextureAspect::All,
    },
    data,
    layout,
    size,
  );
}
