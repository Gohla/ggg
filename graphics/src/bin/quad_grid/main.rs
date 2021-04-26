///! Quad grids

use wgpu::{CommandBuffer, PowerPreference, RenderPipeline};

use app::{Frame, Gfx, GuiFrame, Options, Os};
use common::input::RawInput;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use graphics::include_shader;
use gfx::texture_def::ArrayTextureDefBuilder;

#[derive(Default)]
pub struct Input {}

pub struct QuadGrid {

  render_pipeline: RenderPipeline,
}

impl app::Application for QuadGrid {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let mut texture_def_builder = ArrayTextureDefBuilder::new(350, 350);
    let texture_1 = texture_def_builder.add_texture(image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/cobble_stone_1.png")).unwrap()).unwrap();
    let texture_2 = texture_def_builder.add_texture(image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/concrete_1_1.png")).unwrap()).unwrap();
    let texture_def = texture_def_builder.build(
      &gfx.device,
      &gfx.queue,
      "Quad grid array texture",
      "Quad grid array texture view",
      "Quad grid array texture sampler",
      "Quad grid array texture bind group layout",
      "Quad grid array texture bind group",
    );
    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_layout_label("Quad grid pipeline layout")
      .with_label("Quad grid render pipeline")
      .build(&gfx.device);

    Self {
      render_pipeline,
    }
  }


  type Input = Input;

  fn process_input(&mut self, _raw_input: RawInput) -> Input {
    let input = Input::default();
    input
  }


  fn render<'a>(&mut self, _os: &Os, _gfx: &Gfx, frame: Frame<'a>, _gui_frame: &GuiFrame, _input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut render_pass = RenderPassBuilder::new()
      .with_label("Quad grid render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Quad grid");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.draw(0..0, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<QuadGrid>(Options {
    name: "Quad grid".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
