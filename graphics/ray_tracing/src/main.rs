use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec2;
///! GPU ray tracing in one weekend: https://raytracing.github.io/books/RayTracingInOneWeekend.html + http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/

use wgpu::{BindGroup, CommandBuffer, include_spirv, RenderPipeline, ShaderStage};

use app::{Frame, Gfx, GuiFrame, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  viewport: Vec2,
}

impl Uniform {
  pub fn from_screen_size(screen_size: ScreenSize) -> Self {
    Self {
      viewport: Vec2::new(screen_size.physical.width as f32, screen_size.physical.height as f32)
    }
  }
}

pub struct RayTracing {
  uniform_buffer: GfxBuffer,
  static_bind_group: BindGroup,

  render_pipeline: RenderPipeline,
}

impl app::Application for RayTracing {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Ray tracing uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_screen_size(os.window.get_inner_size())]);
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
      .with_layout_label("Ray tracing pipeline layout")
      .with_label("Ray tracing render pipeline")
      .build(&gfx.device);

    Self {
      uniform_buffer,
      static_bind_group,

      render_pipeline,
    }
  }

  type Input = ();

  fn process_input(&mut self, _raw_input: RawInput) -> () {}

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, _gui_frame: &GuiFrame, _input: &()) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::from_screen_size(frame.screen_size)]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Ray tracing render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Trace rays");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    render_pass.draw(0..3, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() { app::run_with_defaults::<RayTracing>("Ray tracing").unwrap(); }
