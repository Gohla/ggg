///! GPU ray tracing in one weekend: https://raytracing.github.io/books/RayTracingInOneWeekend.html + http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, RenderPipeline, ShaderStages};

use app::{AppRunner, RenderInput};
use common::input::{KeyboardKey, KeyboardModifier, RawInput};
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::full_screen_triangle::FullScreenTriangle;
use gfx::render_pass::RenderPassBuilder;
use os::Os;

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

  fn new(_os: &Os, gfx: &Gfx, screen_size: ScreenSize, _config: Self::Config) -> Self {
    let camera_aperture = 0.1;
    let camera_origin = Vec3::new(0.0, 1.0, 3.0);
    let v_fov = 45.0;
    let uniform_buffer = BufferBuilder::new()
      .uniform_usage()
      .label("Ray tracing uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::new(screen_size, 0.0, camera_aperture, camera_origin, v_fov)]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::FRAGMENT);

    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Ray tracing static bind group layout")
      .with_label("Ray tracing static bind group")
      .build(&gfx.device);

    let full_screen_triangle = FullScreenTriangle::new(&gfx.device);
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));
    let (_, render_pipeline) = full_screen_triangle.create_render_pipeline_builder(&gfx)
      .layout_label("Ray tracing pipeline layout")
      .bind_group_layouts(&[&static_bind_group_layout])
      .label("Ray tracing render pipeline")
      .fragment_module(&fragment_shader_module)
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
    if raw_input.is_keyboard_key_down(KeyboardKey::KeyW) {
      input.forward = true;
    }
    if raw_input.is_keyboard_key_down(KeyboardKey::KeyS) {
      input.backward = true;
    }
    if raw_input.is_keyboard_key_down(KeyboardKey::KeyA) {
      input.left = true;
    }
    if raw_input.is_keyboard_key_down(KeyboardKey::KeyD) {
      input.right = true;
    }
    if raw_input.is_keyboard_key_down(KeyboardKey::Space) {
      input.up = true;
    }
    if raw_input.is_keyboard_key_down(KeyboardKey::KeyC) {
      input.down = true;
    }
    if raw_input.is_keyboard_modifier_down(KeyboardModifier::Control) {
      input.aperture_delta = (raw_input.mouse_wheel_pixel_delta.physical.y as f32 + raw_input.mouse_wheel_line_delta.y as f32) * 0.1;
    } else {
      input.v_fov_delta = raw_input.mouse_wheel_pixel_delta.physical.y as f32 + raw_input.mouse_wheel_line_delta.y as f32;
    }
    input
  }


  fn render<'a>(&mut self, RenderInput { gfx, frame, elapsed, input, mut render, .. }: RenderInput<'a, Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let duration = frame.duration.into_seconds() as f32;
    if input.forward { self.camera_origin.z -= 1.0 * duration; }
    if input.backward { self.camera_origin.z += 1.0 * duration; }
    if input.left { self.camera_origin.x += 1.0 * duration; }
    if input.right { self.camera_origin.x -= 1.0 * duration; }
    if input.up { self.camera_origin.y += 1.0 * duration; }
    if input.down { self.camera_origin.y -= 1.0 * duration; }
    self.v_fov += input.v_fov_delta;
    self.camera_aperture += input.aperture_delta;
    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform::new(render.screen_size, elapsed.into_seconds() as f32, self.camera_aperture, self.camera_origin, self.v_fov)]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Ray tracing render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, &mut render, false);
    render_pass.push_debug_group("Trace rays");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    self.full_screen_triangle.draw(&mut render_pass);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Ray tracing")
    .with_high_power_graphics_adapter()
    .without_depth_stencil_texture()
    .run::<RayTracing>()
    .unwrap();
}
