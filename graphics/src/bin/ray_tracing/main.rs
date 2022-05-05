///! GPU ray tracing in one weekend: https://raytracing.github.io/books/RayTracingInOneWeekend.html + http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, Features, PowerPreference, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::input::{KeyboardButton, KeyboardModifier, RawInput};
use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader_without_validation_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::full_screen_triangle::FullScreenTriangle;
use gfx::render_pass::RenderPassBuilder;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  viewport_and_elapsed_and_aperture: Vec4,
  camera_origin_and_v_fov: Vec4,
}

impl Uniform {
  pub fn new(screen_size: ScreenSize, elapsed: f32, camera_aperture: f32, camera_origin: Vec3, v_fov: f32) -> Self {
    Self {
      viewport_and_elapsed_and_aperture: Vec4::new(screen_size.physical.width as f32, screen_size.physical.height as f32, elapsed, camera_aperture),
      camera_origin_and_v_fov: Vec4::new(camera_origin.x, camera_origin.y, camera_origin.z, v_fov),
    }
  }
}

#[derive(Default)]
pub struct Input {
  forward: bool,
  backward: bool,
  up: bool,
  down: bool,
  left: bool,
  right: bool,
  v_fov_delta: f32,
  aperture_delta: f32,
}

pub struct RayTracing {
  uniform_buffer: GfxBuffer,
  static_bind_group: BindGroup,

  full_screen_triangle: FullScreenTriangle,
  render_pipeline: RenderPipeline,

  camera_aperture: f32,
  camera_origin: Vec3,
  v_fov: f32,
}

impl app::Application for RayTracing {
  type Config = ();

  fn new(os: &Os, gfx: &Gfx, _config: Self::Config) -> Self {
    let camera_aperture = 0.1;
    let camera_origin = Vec3::new(0.0, 1.0, 3.0);
    let v_fov = 45.0;
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Ray tracing uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::new(os.window.get_inner_size(), 0.0, camera_aperture, camera_origin, v_fov)]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::FRAGMENT);

    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Ray tracing static bind group layout")
      .with_label("Ray tracing static bind group")
      .build(&gfx.device);

    let full_screen_triangle = FullScreenTriangle::new(&gfx.device);
    let fragment_shader_module = unsafe { gfx.device.create_shader_module_spirv(&include_shader_without_validation_for_bin!("frag")) };
    let (_, render_pipeline) = full_screen_triangle.create_render_pipeline_builder()
      .with_bind_group_layouts(&[&static_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_layout_label("Ray tracing pipeline layout")
      .with_label("Ray tracing render pipeline")
      .build(&gfx.device);

    Self {
      uniform_buffer,
      static_bind_group,

      full_screen_triangle,
      render_pipeline,

      camera_aperture,
      camera_origin,
      v_fov,
    }
  }


  type Input = Input;

  fn process_input(&mut self, raw_input: RawInput) -> Input {
    let mut input = Input::default();
    if raw_input.is_keyboard_button_down(KeyboardButton::W) {
      input.forward = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::S) {
      input.backward = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::A) {
      input.left = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::D) {
      input.right = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::Space) {
      input.up = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::C) {
      input.down = true;
    }
    if raw_input.is_keyboard_modifier_down(KeyboardModifier::Control) {
      input.aperture_delta = (raw_input.mouse_wheel_pixel_delta.physical.y as f32 + raw_input.mouse_wheel_line_delta.vertical as f32) * 0.1;
    } else {
      input.v_fov_delta = raw_input.mouse_wheel_pixel_delta.physical.y as f32 + raw_input.mouse_wheel_line_delta.vertical as f32;
    }
    input
  }


  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, _gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let delta = frame.time.delta.as_s() as f32;
    if input.forward { self.camera_origin.z -= 1.0 * delta; }
    if input.backward { self.camera_origin.z += 1.0 * delta; }
    if input.left { self.camera_origin.x += 1.0 * delta; }
    if input.right { self.camera_origin.x -= 1.0 * delta; }
    if input.up { self.camera_origin.y += 1.0 * delta; }
    if input.down { self.camera_origin.y -= 1.0 * delta; }
    self.v_fov += input.v_fov_delta;
    self.camera_aperture += input.aperture_delta;
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::new(frame.screen_size, frame.time.elapsed.as_s() as f32, self.camera_aperture, self.camera_origin, self.v_fov)]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Ray tracing render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, &mut frame, false);
    render_pass.push_debug_group("Trace rays");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    self.full_screen_triangle.draw(&mut render_pass);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<RayTracing>(Options {
    name: "Ray tracing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    require_graphics_device_features: Features::SPIRV_SHADER_PASSTHROUGH,
    depth_stencil_texture_format: None,
    ..Options::default()
  }).unwrap();
}
