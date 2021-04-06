use bytemuck::{Pod, Zeroable};
use egui::{CtxRef, Event, Key, PointerButton, Pos2, RawInput as EguiRawInput, Vec2};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendFactor, BlendOperation, BlendState, Color, ColorTargetState, Device, FilterMode, InputStepMode, PipelineLayout, RenderPipeline, ShaderStage, TextureFormat, VertexBufferLayout};

use common::input::{KeyboardButton, KeyboardModifier, MouseButton, RawInput};
use common::screen::ScreenSize;
use gfx::bind_group::{BindGroupLayoutBuilder, CombinedBindGroupLayoutBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::prelude::*;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;

pub struct Gui {
  context: CtxRef,
  input: EguiRawInput,
}

impl Gui {
  pub fn new() -> Self {
    Self {
      context: CtxRef::default(),
      input: EguiRawInput::default(),
    }
  }

  pub fn process_input(&mut self, input: RawInput) {
    // Mouse wheel delta
    self.input.scroll_delta = Vec2::new( // TODO: properly handle line to pixel conversion?
                                         (input.mouse_wheel_pixel_delta.horizontal + input.mouse_wheel_line_delta.horizontal * 24.0) as f32,
                                         (input.mouse_wheel_pixel_delta.vertical + input.mouse_wheel_line_delta.vertical * 24.0) as f32,
    );

    // Keyboard modifiers
    if input.is_keyboard_modifier_down(KeyboardModifier::Shift) {
      self.input.modifiers.shift = true;
    }
    if input.is_keyboard_modifier_down(KeyboardModifier::Control) {
      self.input.modifiers.ctrl = true;
      #[cfg(not(target_os = "macos"))] { self.input.modifiers.command = true; }
    }
    if input.is_keyboard_modifier_down(KeyboardModifier::Alternate) {
      self.input.modifiers.alt = true;
    }
    #[cfg(target_os = "macos")]
    if input.is_keyboard_modifier_down(KeyboardModifier::Meta) {
      self.input.modifiers.mac_cmd = true;
      self.input.modifiers.command = true;
    }
    let modifiers = self.input.modifiers;

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

        _ => {
          return None;
        }
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

  pub fn begin_frame(&mut self, screen_size: ScreenSize, elapsed_seconds: f64, delta_seconds: f64) -> egui::CtxRef {
    self.input.screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(screen_size.physical.width as f32, screen_size.physical.height as f32)));
    let pixels_per_point: f64 = screen_size.scale.into();
    self.input.pixels_per_point = Some(pixels_per_point as f32);

    self.input.time = Some(elapsed_seconds);
    self.input.predicted_dt = delta_seconds as f32;

    let input = std::mem::take(&mut self.input);
    self.context.begin_frame(input);
    self.context.clone()
  }

  pub fn end_frame(&self) {
    let (output, shapes) = self.context.end_frame();
    let clipped_meshes = self.context.tessellate(shapes);
    // let wants_keyboard_input = self.context.wants_keyboard_input();
    // let wants_pointer_input = self.context.wants_pointer_input();
  }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, Pod, Zeroable)]
struct GuiUniform {
  screen_size: [f32; 2],
}

pub struct GuiRenderPass {
  index_buffers: Vec<GfxBuffer>,
  vertex_buffers: Vec<GfxBuffer>,
  uniform_buffer: GfxBuffer,
  static_bind_group_layout: BindGroupLayout,
  static_bind_group: BindGroup,
  dynamic_bind_group_layout: BindGroupLayout,
  dynamic_bind_group: Option<BindGroup>,
  pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
}

impl GuiRenderPass {
  pub fn new(device: &Device, swap_chain_texture_format: TextureFormat) -> Self {
    let vertex_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/gui.vert.spv"));
    let fragment_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/gui.frag.spv"));

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("GUI uniform buffer")
      .build_with_data(device, &[GuiUniform::default()]);
    let (uniform_buffer_bind_layout_entry, uniform_buffer_bind_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStage::VERTEX);
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
    let dynamic_bind_group_layout = BindGroupLayoutBuilder::new()
      .with_entries(&[BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Texture {
          multisampled: false,
          sample_type: wgpu::TextureSampleType::Float { filterable: true },
          view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
      }])
      .with_label("GUI dynamic bind group layout")
      .build(device);
    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&static_bind_group_layout, &dynamic_bind_group_layout])
      .with_vertex_buffer_layouts(&[VertexBufferLayout {
        array_stride: 5 * 4,
        step_mode: InputStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2 => Uint],
      }])
      .with_fragment_state(&fragment_shader_module, "main", &[ColorTargetState {
        format: swap_chain_texture_format,
        alpha_blend: BlendState {
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
      index_buffers: Vec::with_capacity(64),
      vertex_buffers: Vec::with_capacity(64),
      uniform_buffer,
      static_bind_group_layout,
      static_bind_group,
      dynamic_bind_group_layout,
      dynamic_bind_group: None,
      pipeline_layout,
      render_pipeline,
    }
  }
}
