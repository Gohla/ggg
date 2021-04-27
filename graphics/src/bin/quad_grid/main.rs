use std::mem::size_of;

///! Quad grids

use bytemuck::{Pod, Zeroable};
use ultraviolet::Mat4;
use wgpu::{BindGroup, BufferAddress, CommandBuffer, PowerPreference, RenderPipeline, ShaderStage};

use app::{Frame, Gfx, GuiFrame, Options, Os};
use common::input::RawInput;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::CameraSys;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture_def::{ArrayTextureDef, ArrayTextureDefBuilder, TextureIdx};
use graphics::include_shader;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}

impl Uniform {
  pub fn from_camera_sys(camera_sys: &CameraSys) -> Self {
    Self {
      view_projection: camera_sys.get_view_projection_matrix(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  texture_indexes: [u32; 4],
}

#[derive(Default)]
pub struct Input {}

pub struct QuadGrid {
  camera_sys: CameraSys,

  uniform_buffer: GfxBuffer,
  instance_buffer: GfxBuffer,
  bind_group: BindGroup,

  array_texture_def: ArrayTextureDef,

  render_pipeline: RenderPipeline,
}

impl app::Application for QuadGrid {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let viewport = gfx.swap_chain.get_size().physical;
    let mut camera_sys = CameraSys::with_defaults_perspective(viewport);

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Quad grid uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_camera_sys(&camera_sys)]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStage::VERTEX);

    let instance_buffer = {
      let buffer = BufferBuilder::new()
        .with_size((1 * size_of::<Instance>()) as BufferAddress)
        .with_storage_usage()
        .with_mapped_at_creation(true)
        .with_label("Quad grid instance storage buffer")
        .build(&gfx.device);
      // {
      //   let mut view = buffer.slice(..).get_mapped_range_mut();
      //   let instance_slice: &mut [Instance] = bytemuck::cast_slice_mut(&mut view);
      //   (0..num_cubes_to_generate)
      //     .map(|_| Instance::from_random_range(&mut rng, -cube_position_range..cube_position_range))
      //     .zip(instance_slice)
      //     .for_each(|(instance, place)| *place = instance);
      // }
      buffer.unmap();
      buffer
    };
    let (instance_bind_group_layout_entry, instance_bind_group_entry) = instance_buffer.create_storage_binding_entries(1, ShaderStage::VERTEX, true);

    let (bind_group_layout, bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry, instance_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry, instance_bind_group_entry])
      .with_layout_label("Quad grid bind group layout")
      .with_label("Quad grid bind group")
      .build(&gfx.device);

    let mut array_texture_def_builder = ArrayTextureDefBuilder::new(350, 350);
    let texture_1 = array_texture_def_builder.add_texture(image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/cobble_stone_1.png")).unwrap()).unwrap();
    let texture_2 = array_texture_def_builder.add_texture(image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/concrete_1_1.png")).unwrap()).unwrap();
    let array_texture_def = array_texture_def_builder.build(
      &gfx.device,
      &gfx.queue,
      "Quad grid array texture",
      "Quad grid array texture view",
      "Quad grid array texture sampler",
      "Quad grid array texture bind group layout",
      "Quad grid array texture bind group",
    ).unwrap();

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&bind_group_layout, &array_texture_def.bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_layout_label("Quad grid pipeline layout")
      .with_label("Quad grid render pipeline")
      .build(&gfx.device);

    Self {
      camera_sys,
      uniform_buffer,
      instance_buffer,
      bind_group,
      array_texture_def,
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
    render_pass.set_bind_group(0, &self.bind_group, &[]);
    render_pass.set_bind_group(1, &self.array_texture_def.bind_group, &[]);
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
