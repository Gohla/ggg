///! Quad grids

use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use rand::prelude::*;
use ultraviolet::Mat4;
use wgpu::{BufferAddress, CommandBuffer, IndexFormat, RenderPipeline, ShaderStages};

use app::{AppRunner, RenderInput};
use common::idx_assigner::Item;
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraInput, CameraSettings};
use gfx::camera::debug::CameraDebugging;
use gfx::texture_def::{ArrayTextureDef, ArrayTextureDefBuilder};
use os::Os;

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
  pub fn from_camera(camera: &Camera) -> Self {
    Self {
      view_projection: camera.get_view_projection_matrix(),
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
  camera_settings: CameraSettings,
  camera_debugging: CameraDebugging,
  camera: Camera,

  uniform_buffer: GfxBuffer,
  _instance_buffer: GfxBuffer,
  bind_group: CombinedBindGroup,

  array_texture_def: ArrayTextureDef,

  render_pipeline: RenderPipeline,

  index_buffer: GfxBuffer,
}

impl app::Application for QuadGrid {
  type Config = ();
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, _config: Self::Config) -> Self {
    let mut camera_settings = CameraSettings::with_defaults_arcball_perspective();
    let camera_debugging = CameraDebugging::with_default_settings(camera_settings);
    let camera = Camera::new(viewport.physical, &mut camera_settings);

    let uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Quad grid uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_camera(&camera)]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::VERTEX);

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
    );

    let mut rng = SmallRng::seed_from_u64(101702198783735);
    let instance_buffer = {
      let buffer = BufferBuilder::default()
        .size((1 * size_of::<Instance>()) as BufferAddress)
        .storage_usage()
        .mapped_at_creation(true)
        .label("Quad grid instance storage buffer")
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
    let instance_binding = instance_buffer.binding(1, ShaderStages::VERTEX);

    let bind_group = CombinedBindGroupBuilder::default()
      .layout_entries(&[uniform_binding.layout, instance_binding.layout])
      .entries(&[uniform_binding.entry, instance_binding.entry])
      .layout_label("Quad grid bind group layout")
      .label("Quad grid bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));
    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Quad grid pipeline layout")
      .bind_group_layouts(&[&bind_group.layout, &array_texture_def.bind_group.layout])
      .label("Quad grid render pipeline")
      .vertex_module(&vertex_shader_module)
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);

    let index_buffer = {
      let data: Vec<_> = (0..MAX_INDICES).map(|i| {
        let quad = i / NUM_QUAD_INDICES;
        let quad_local = i % NUM_QUAD_INDICES;
        QUAD_INDICES[quad_local] + quad as u32 * NUM_QUAD_VERTICES as u32
      }).collect();
      BufferBuilder::default()
        .static_index_usage()
        .label("Quad grid static index buffer")
        .build_with_data(&gfx.device, &data)
    };

    Self {
      camera_settings,
      camera_debugging,
      camera,
      uniform_buffer,
      _instance_buffer: instance_buffer,
      bind_group,
      array_texture_def,
      render_pipeline,
      index_buffer,
    }
  }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    self.camera.set_viewport(viewport.physical);
  }

  type Input = Input;
  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.camera_debugging.add_to_menu(ui);
  }

  fn render(&mut self, RenderInput { gfx, frame, input, gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_debugging.show_single(&gui, &self.camera, &mut self.camera_settings);
    self.camera.update(&mut self.camera_settings, &input.camera, frame.duration);
    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform::from_camera(&self.camera)]);

    let mut pass = gfx_frame.render_pass_builder().label("Quad grid render pass").begin();
    pass.push_debug_group("Quad grid");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.bind_group.entry, &[]);
    pass.set_bind_group(1, &self.array_texture_def.bind_group.entry, &[]);
    pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    pass.draw_indexed(0..MAX_INDICES as u32, 0, 0..1);
    pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Quad grid")
    .without_depth_stencil_texture()
    .run::<QuadGrid>()
    .unwrap();
}
