use bytemuck::{Pod, Zeroable};
use ultraviolet::{Vec2, Vec3};
///! GPU ray tracing in one weekend: https://raytracing.github.io/books/RayTracingInOneWeekend.html + http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/

use wgpu::{BindGroup, CommandBuffer, include_spirv, PowerPreference, RenderPipeline, ShaderStage};

use app::{Frame, Gfx, GuiFrame, Options, Os};
use common::input::{RawInput, KeyboardButton};
use common::screen::ScreenSize;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  viewport: Vec2,
  elapsed: f32,
  camera_origin: Vec3,
}

impl Uniform {
  pub fn new(screen_size: ScreenSize, elapsed: f32, camera_origin: Vec3) -> Self {
    Self {
      viewport: Vec2::new(screen_size.physical.width as f32, screen_size.physical.height as f32),
      elapsed,
      camera_origin,
    }
  }
}

#[derive(Default)]
pub struct Input {
  up: bool,
  down: bool,
  left: bool,
  right: bool,
  scroll: f32,
}

pub struct RayTracing {
  uniform_buffer: GfxBuffer,
  static_bind_group: BindGroup,

  render_pipeline: RenderPipeline,
  multisampled_framebuffer: GfxTexture,

  camera_origin: Vec3,
}

const SAMPLE_COUNT: u32 = 1;

impl app::Application for RayTracing {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Ray tracing uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::new(os.window.get_inner_size(), 0.0, Vec3::zero())]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStage::FRAGMENT);

    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Ray tracing static bind group layout")
      .with_label("Ray tracing static bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!(concat!(env!("OUT_DIR"), "/shader/ray_tracing.vert.spv")));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!(concat!(env!("OUT_DIR"), "/shader/ray_tracing.frag.spv")));
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&static_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_multisample_count(SAMPLE_COUNT)
      .with_layout_label("Ray tracing pipeline layout")
      .with_label("Ray tracing render pipeline")
      .build(&gfx.device);

    let multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.swap_chain, SAMPLE_COUNT)
      .with_texture_label("Multisampling texture")
      .with_texture_view_label("Multisampling texture view")
      .build(&gfx.device);

    Self {
      uniform_buffer,
      static_bind_group,

      render_pipeline,

      multisampled_framebuffer,

      camera_origin: Vec3::zero(),
    }
  }

  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, _screen_size: ScreenSize) {
    self.multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.swap_chain, SAMPLE_COUNT)
      .build(&gfx.device);
  }


  type Input = Input;

  fn process_input(&mut self, raw_input: RawInput) -> Input {
    let mut input = Input::default();
    if raw_input.is_keyboard_button_down(KeyboardButton::W) {
      input.up = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::S) {
      input.down = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::A) {
      input.left = true;
    }
    if raw_input.is_keyboard_button_down(KeyboardButton::D) {
      input.right = true;
    }
    input.scroll = raw_input.mouse_wheel_pixel_delta.physical.y as f32 + raw_input.mouse_wheel_line_delta.vertical as f32;
    input.scroll *= 0.01;
    input
  }


  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, _gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let delta = frame.time.delta.as_s() as f32;
    if input.up { self.camera_origin.y += 1.0 * delta; }
    if input.down { self.camera_origin.y -= 1.0 * delta; }
    if input.left { self.camera_origin.x += 1.0 * delta; }
    if input.right { self.camera_origin.x -= 1.0 * delta; }
    self.camera_origin.z += input.scroll;
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::new(frame.screen_size, frame.time.elapsed.as_s() as f32, self.camera_origin)]);

    let render_pass_builder = RenderPassBuilder::new()
      .with_label("Ray tracing render pass");
    let mut render_pass = if SAMPLE_COUNT != 1 {
      render_pass_builder.begin_render_pass_for_multisampled_swap_chain_with_clear(frame.encoder, &self.multisampled_framebuffer.view, &frame.output_texture)
    } else {
      render_pass_builder.begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture)
    };
    render_pass.push_debug_group("Trace rays");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    render_pass.draw(0..3, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<RayTracing>(Options {
    name: "Ray tracing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
