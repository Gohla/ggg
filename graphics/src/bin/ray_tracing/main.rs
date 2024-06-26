///! GPU ray tracing in one weekend: https://raytracing.github.io/books/RayTracingInOneWeekend.html + http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Vec3, Vec4};
use wgpu::{CommandBuffer, RenderPipeline, ShaderStages};

use app::{AppRunner, RenderInput};
use common::input::{KeyboardKey, KeyboardModifier, RawInput};
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::full_screen_triangle::FullScreenTriangle;
use os::Os;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  viewport_and_elapsed_and_aperture: Vec4,
  camera_origin_and_v_fov: Vec4,
}

impl Uniform {
  pub fn new(viewport: ScreenSize, elapsed: f32, camera_aperture: f32, camera_origin: Vec3, v_fov: f32) -> Self {
    Self {
      viewport_and_elapsed_and_aperture: Vec4::new(viewport.physical.width as f32, viewport.physical.height as f32, elapsed, camera_aperture),
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
  static_bind_group: CombinedBindGroup,

  full_screen_triangle: FullScreenTriangle,
  render_pipeline: RenderPipeline,

  camera_aperture: f32,
  camera_origin: Vec3,
  v_fov: f32,
}

impl app::Application for RayTracing {
  type Data = ();
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, _config: Self::Data) -> Self {
    let camera_aperture = 0.1;
    let camera_origin = Vec3::new(0.0, 1.0, 3.0);
    let v_fov = 45.0;
    let uniform_buffer = BufferBuilder::default()
      .label("Ray tracing uniform buffer")
      .uniform_usage()
      .build_with_data(&gfx.device, &[Uniform::new(viewport, 0.0, camera_aperture, camera_origin, v_fov)]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::FRAGMENT);

    let static_bind_group = CombinedBindGroupBuilder::default()
      .layout_label("Ray tracing static bind group layout")
      .layout_entries(&[uniform_binding.layout])
      .label("Ray tracing static bind group")
      .entries(&[uniform_binding.entry])
      .build(&gfx.device);

    let full_screen_triangle = FullScreenTriangle::new(&gfx.device);
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));
    let (_, render_pipeline) = full_screen_triangle.create_render_pipeline_builder(&gfx)
      .layout_label("Ray tracing pipeline layout")
      .bind_group_layouts(&[&static_bind_group.layout])
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

  fn render(&mut self, RenderInput { gfx, frame, elapsed, input, gfx_frame, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let duration = frame.duration.into_seconds() as f32;
    if input.forward { self.camera_origin.z -= 1.0 * duration; }
    if input.backward { self.camera_origin.z += 1.0 * duration; }
    if input.left { self.camera_origin.x += 1.0 * duration; }
    if input.right { self.camera_origin.x -= 1.0 * duration; }
    if input.up { self.camera_origin.y += 1.0 * duration; }
    if input.down { self.camera_origin.y -= 1.0 * duration; }
    self.v_fov += input.v_fov_delta;
    self.camera_aperture += input.aperture_delta;
    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform::new(gfx_frame.viewport, elapsed.into_seconds() as f32, self.camera_aperture, self.camera_origin, self.v_fov)]);

    let mut pass = gfx_frame.render_pass_builder()
      .label("Ray tracing render pass")
      .begin();
    pass.push_debug_group("Trace rays");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.static_bind_group.entry, &[]);
    self.full_screen_triangle.draw(&mut pass);
    pass.pop_debug_group();
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
