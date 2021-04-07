use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{ClippedMesh, CtxRef, Event, Key, PointerButton, Pos2, RawInput as EguiRawInput, Rect, Vec2};
use egui::epaint::{Mesh, Vertex};
use wgpu::{BindGroup, BindGroupLayout, BlendFactor, BlendOperation, BlendState, BufferAddress, ColorTargetState, CommandEncoder, Device, FilterMode, IndexFormat, InputStepMode, PipelineLayout, Queue, RenderPipeline, ShaderStage, SwapChainTexture, TextureFormat, VertexBufferLayout};

use common::input::{KeyboardButton, KeyboardModifier, MouseButton, RawInput};
use common::screen::ScreenSize;
use gfx::bind_group::{BindGroupBuilder, BindGroupLayoutBuilder, BindGroupLayoutEntryBuilder, CombinedBindGroupLayoutBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::prelude::*;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::TextureBuilder;

pub struct Gui {
  context: CtxRef,
  input: EguiRawInput,

  index_buffer: Option<GfxBuffer>,
  vertex_buffer: Option<GfxBuffer>,
  uniform_buffer: GfxBuffer,
  _static_bind_group_layout: BindGroupLayout,
  static_bind_group: BindGroup,
  texture_bind_group_layout: BindGroupLayout,
  texture_bind_group: Option<BindGroup>,
  previous_texture_version: u64,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
}

// Creation

impl Gui {
  pub fn new(device: &Device, swap_chain_texture_format: TextureFormat) -> Self {
    let vertex_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/gui.vert.spv"));
    let fragment_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/gui.frag.spv"));
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("GUI uniform buffer")
      .build_with_data(device, &[Uniform::default()]);
    let (uniform_buffer_bind_layout_entry, uniform_buffer_bind_entry) =
      uniform_buffer.create_uniform_binding_entries(0, ShaderStage::VERTEX);
    let sampler = SamplerBuilder::new()
      .with_mag_filter(FilterMode::Linear)
      .with_min_filter(FilterMode::Linear)
      .with_label("GUI texture sampler")
      .build(device);
    let (sampler_bind_layout_entry, sampler_bind_entry) = sampler.create_bind_group_entries(1, ShaderStage::FRAGMENT);
    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_buffer_bind_layout_entry, sampler_bind_layout_entry])
      .with_layout_label("GUI static bind group layout")
      .with_entries(&[uniform_buffer_bind_entry, sampler_bind_entry])
      .with_label("GUI static bind group")
      .build(device);
    let texture_bind_group_layout = BindGroupLayoutBuilder::new()
      .with_entries(&[BindGroupLayoutEntryBuilder::new_float_2d_texture()
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
        step_mode: InputStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2 => Uint],
      }])
      .with_fragment_state(&fragment_shader_module, "main", &[ColorTargetState {
        format: swap_chain_texture_format,
        alpha_blend: BlendState { // Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/5f33cf76d952c67bdbe7bd4ed01023899d3ac996/src/lib.rs#L201-L210
          src_factor: BlendFactor::OneMinusDstAlpha,
          dst_factor: BlendFactor::One,
          operation: BlendOperation::Add,
        },
        color_blend: BlendState {
          src_factor: BlendFactor::One,
          dst_factor: BlendFactor::OneMinusSrcAlpha,
          operation: BlendOperation::Add,
        },
        write_mask: Default::default(),
      }])
      .with_layout_label("GUI pipeline layout")
      .with_label("GUI render pipeline")
      .build(device);
    Self {
      context: CtxRef::default(),
      input: EguiRawInput::default(),
      index_buffer: None,
      vertex_buffer: None,
      uniform_buffer,
      _static_bind_group_layout: static_bind_group_layout,
      static_bind_group,
      texture_bind_group_layout,
      texture_bind_group: None,
      previous_texture_version: 0,
      _pipeline_layout: pipeline_layout,
      render_pipeline,
    }
  }
}

// Input processing.

impl Gui {
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
      self.input.scroll_delta = Vec2::new(
        (input.mouse_wheel_pixel_delta.horizontal + input.mouse_wheel_line_delta.horizontal * 24.0) as f32,
        (input.mouse_wheel_pixel_delta.vertical + input.mouse_wheel_line_delta.vertical * 24.0) as f32,
      );

      // Mouse movement
      let mouse_position = Pos2::new(input.mouse_position.x as f32, input.mouse_position.y as f32);
      if !input.mouse_position_delta.is_empty() {
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

// Creating a GUI frame to build the GUI.

pub struct GuiFrame {
  pub context: CtxRef,
}

impl Gui {
  pub fn begin_frame(
    &mut self,
    screen_size: ScreenSize,
    elapsed_seconds: f64,
    delta_seconds: f64,
  ) -> GuiFrame {
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(screen_size.physical.width as f32, screen_size.physical.height as f32));
    self.input.screen_rect = Some(screen_rect);
    let pixels_per_point: f64 = screen_size.scale.into();
    self.input.pixels_per_point = Some(pixels_per_point as f32);

    self.input.time = Some(elapsed_seconds);
    self.input.predicted_dt = delta_seconds as f32;

    let input = std::mem::take(&mut self.input);
    self.context.begin_frame(input);
    GuiFrame { context: self.context.clone() }
  }
}

impl std::ops::Deref for GuiFrame {
  type Target = CtxRef;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.context }
}

// Rendering

impl Gui {
  pub fn render(
    &mut self,
    gui_frame: GuiFrame,
    screen_size: ScreenSize,
    device: &Device,
    queue: &Queue,
    encoder: &mut CommandEncoder,
    swap_chain_texture: &SwapChainTexture,
  ) {
    // Take ownership of the GUI frame and just drop it to prevent further GUI building this frame.
    drop(gui_frame);

    // Update texture if no texture was created yet, or if the texture has changed.
    let texture = self.context.texture();
    if self.texture_bind_group.is_none() || texture.version != self.previous_texture_version {
      self.previous_texture_version = texture.version;
      // Convert into Rgba8UnormSrgb format.
      let mut pixels: Vec<u8> = Vec::with_capacity(texture.pixels.len() * 4);
      for srgba in texture.srgba_pixels() {
        pixels.push(srgba.r());
        pixels.push(srgba.g());
        pixels.push(srgba.b());
        pixels.push(srgba.a());
      }
      // Create and write texture.
      let texture = TextureBuilder::new()
        .with_2d_size(texture.width as u32, texture.height as u32)
        .with_rgba8_unorm_srgb_format()
        .with_sampled_usage()
        .with_texture_label("GUI texture")
        .with_texture_view_label("GUI texture view")
        .build(device);
      texture.write_rgba_texture_data(queue, pixels.as_slice());
      // Create texture bind group.
      let texture_bind_group = BindGroupBuilder::new(&self.texture_bind_group_layout)
        .with_entries(&[texture.create_bind_group_entry(0)])
        .with_label("GUI texture bind group")
        .build(device);
      self.texture_bind_group = Some(texture_bind_group)
    }

    // Get vertices to draw.
    let (_output, shapes) = self.context.end_frame();
    let clipped_meshes: Vec<ClippedMesh> = self.context.tessellate(shapes);

    // (Re-)Create index and vertex buffers if they do not exist yet or are not large enough.
    let index_buffer = {
      let count: usize = clipped_meshes.iter().map(|cm| cm.1.indices.len()).sum();
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
      let count: usize = clipped_meshes.iter().map(|cm| cm.1.vertices.len()).sum();
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
    struct Draw { clip_rect: Rect, indices: Range<u32>, base_vertex: i32 }
    let mut draws = Vec::with_capacity(clipped_meshes.len());
    for ClippedMesh(clip_rect, Mesh { indices, vertices, texture_id: _texture_id }) in &clipped_meshes {
      index_buffer.write_data(queue, index_buffer_offset, indices);
      vertex_buffer.write_bytes(queue, vertex_buffer_offset, as_byte_slice(vertices));
      draws.push(Draw { clip_rect: *clip_rect, indices: index_offset..index_offset + indices.len() as u32, base_vertex: vertex_offset as i32 });
      index_offset += indices.len() as u32;
      index_buffer_offset += (indices.len() * size_of::<u32>()) as BufferAddress;
      vertex_offset += vertices.len();
      vertex_buffer_offset += (vertices.len() * size_of::<Vertex>()) as BufferAddress;
    }

    // Render
    {
      let mut render_pass = RenderPassBuilder::new()
        .with_label("GUI render pass")
        .begin_render_pass_for_swap_chain_with_load(encoder, swap_chain_texture);
      render_pass.push_debug_group("Draw GUI");
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.static_bind_group, &[]);
      render_pass.set_bind_group(1, self.texture_bind_group.as_ref().unwrap(), &[]);
      render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
      render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
      let scale_factor: f64 = screen_size.scale.into();
      let scale_factor = scale_factor as f32;
      let physical_width = screen_size.physical.width;
      let physical_height = screen_size.physical.height;
      for Draw { clip_rect, indices, base_vertex } in draws {
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
        render_pass.draw_indexed(indices, base_vertex as i32, 0..1);
      }
      render_pass.pop_debug_group();
    }

    // Store index and vertex buffers (outside of render pass because it held a reference to these buffers).
    self.index_buffer = Some(index_buffer);
    self.vertex_buffer = Some(vertex_buffer);
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
  screen_size: [f32; 2],
}

impl Uniform {
  #[inline]
  pub fn from_screen_size(screen_size: ScreenSize) -> Self {
    Self { screen_size: [screen_size.physical.width as f32, screen_size.physical.height as f32] }
  }
}

// Needed since we can't use bytemuck for external types.
#[inline]
fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
  let len = slice.len() * std::mem::size_of::<T>();
  let ptr = slice.as_ptr() as *const u8;
  unsafe { std::slice::from_raw_parts(ptr, len) }
}
