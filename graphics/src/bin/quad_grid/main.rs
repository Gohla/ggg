///! Quad grids

use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use rand::prelude::*;
use ultraviolet::Mat4;
use wgpu::{BindGroup, BufferAddress, CommandBuffer, IndexFormat, PowerPreference, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::idx_assigner::Item;
use common::input::RawInput;
use gfx::{Frame, Gfx, include_shader_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{CameraInput, Camera};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture_def::{ArrayTextureDef, ArrayTextureDefBuilder};

const NUM_QUAD_INDICES: usize = 6;
const NUM_QUAD_VERTICES: usize = 4;
const QUAD_INDICES: [u32; NUM_QUAD_INDICES] = [
  0, 3, 2,
  0, 1, 3,
];

const MAX_INSTANCES: usize = 64;
const MAX_INDICES: usize = MAX_INSTANCES * NUM_QUAD_INDICES;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}

impl Uniform {
  pub fn from_camera_sys(camera_sys: &Camera) -> Self {
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

impl Instance {
  fn from_random_range(rng: &mut impl Rng, range: Range<u32>) -> Self {
    Instance {
      texture_indexes: [
        rng.gen_range(range.clone()),
        rng.gen_range(range.clone()),
        rng.gen_range(range.clone()),
        rng.gen_range(range.clone()),
      ]
    }
  }
}


#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

pub struct QuadGrid {
  camera_sys: Camera,

  uniform_buffer: GfxBuffer,
  _instance_buffer: GfxBuffer,
  bind_group: BindGroup,

  array_texture_def: ArrayTextureDef,

  render_pipeline: RenderPipeline,

  index_buffer: GfxBuffer,
}

impl app::Application for QuadGrid {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let viewport = gfx.surface.get_size().physical;
    let camera_sys = Camera::with_defaults_arcball_perspective(viewport);

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Quad grid uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_camera_sys(&camera_sys)]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX);

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

    let mut rng = SmallRng::seed_from_u64(101702198783735);
    let instance_buffer = {
      let buffer = BufferBuilder::new()
        .with_size((1 * size_of::<Instance>()) as BufferAddress)
        .with_storage_usage()
        .with_mapped_at_creation(true)
        .with_label("Quad grid instance storage buffer")
        .build(&gfx.device);
      {
        let mut view = buffer.slice(..).get_mapped_range_mut();
        let instance_slice: &mut [Instance] = bytemuck::cast_slice_mut(&mut view);
        (0..MAX_INSTANCES)
          .map(|_| Instance::from_random_range(&mut rng, texture_1.into_idx() as u32..texture_2.into_idx() as u32))
          .zip(instance_slice)
          .for_each(|(instance, place)| *place = instance);
      }
      buffer.unmap();
      buffer
    };
    let (instance_bind_group_layout_entry, instance_bind_group_entry) = instance_buffer.create_storage_binding_entries(1, ShaderStages::VERTEX, true);

    let (bind_group_layout, bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry, instance_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry, instance_bind_group_entry])
      .with_layout_label("Quad grid bind group layout")
      .with_label("Quad grid bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("frag"));
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&bind_group_layout, &array_texture_def.bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_layout_label("Quad grid pipeline layout")
      .with_label("Quad grid render pipeline")
      .build(&gfx.device);

    let index_buffer = {
      let data: Vec<_> = (0..MAX_INDICES).map(|i| {
        let quad = i / NUM_QUAD_INDICES;
        let quad_local = i % NUM_QUAD_INDICES;
        QUAD_INDICES[quad_local] + quad as u32 * NUM_QUAD_VERTICES as u32
      }).collect();
      BufferBuilder::new()
        .with_static_index_usage()
        .with_label("Quad grid static index buffer")
        .build_with_data(&gfx.device, &data)
    };

    Self {
      camera_sys,
      uniform_buffer,
      _instance_buffer: instance_buffer,
      bind_group,
      array_texture_def,
      render_pipeline,
      index_buffer,
    }
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera_sys.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta, &gui_frame);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::from_camera_sys(&self.camera_sys)]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Quad grid render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Quad grid");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.bind_group, &[]);
    render_pass.set_bind_group(1, &self.array_texture_def.bind_group, &[]);
    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    render_pass.draw_indexed(0..MAX_INDICES as u32, 0, 0..1);
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
