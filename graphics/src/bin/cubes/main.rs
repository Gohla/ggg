///! Rendering a lot of generic cubes. Based on https://twitter.com/SebAaltonen/status/1315982782439591938.

use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{DragValue, Ui};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BufferAddress, CommandBuffer, IndexFormat, RenderPipeline, ShaderStages};

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraData, CameraSettings};
use gfx::camera::controller::CameraControllerInput;
use gfx::camera::inspector::CameraInspector;
use gui::widget::UiWidgetsExt;
use os::Os;

const NUM_CUBE_INDICES: usize = 3 * 3 * 2;
const NUM_CUBE_VERTICES: usize = 8;
const CUBE_INDICES: [u32; NUM_CUBE_INDICES] = [
  0, 2, 1, 2, 3, 1,
  5, 4, 1, 1, 4, 0,
  0, 4, 6, 0, 6, 2,
];

const MAX_INSTANCES: usize = 3_500_000;
const MAX_INDICES: usize = MAX_INSTANCES * NUM_CUBE_INDICES;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  camera_position: Vec4,
  view_projection: Mat4,
}
impl Uniform {
  pub fn from_camera(camera: &Camera) -> Self {
    Self {
      camera_position: camera.position().into_homogeneous_point(),
      view_projection: *camera.view_projection_matrix(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  position: Vec4,
}
impl Instance {
  fn from_position(position: Vec3) -> Self { Self { position: position.into_homogeneous_point() } }

  fn from_random_range(rng: &mut impl Rng, range: Range<f32>) -> Self {
    Self::from_position(Vec3::new(rng.gen_range(range.clone()), rng.gen_range(range.clone()), rng.gen_range(range)))
  }
}

pub struct Cubes {
  data: Data,

  camera: Camera,

  uniform_buffer: GfxBuffer,
  instance_buffer: GfxBuffer,
  static_bind_group: CombinedBindGroup,

  render_pipeline: RenderPipeline,

  index_buffer: GfxBuffer,

  rng: SmallRng,

  num_cubes: u32,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  camera_data: CameraData,
  camera_settings: CameraSettings,
  camera_inspector: CameraInspector,

  num_cubes_to_generate: u32,
  cube_position_range: f32,
}
impl Default for Data {
  fn default() -> Self {
    let mut data = Self {
      camera_data: Default::default(),
      camera_settings: Default::default(),
      camera_inspector: Default::default(),
      num_cubes_to_generate: 100_000,
      cube_position_range: 1000.0,
    };
    data.camera_settings.controller.arcball.mouse_scroll_distance_speed = 100.0;

    data.camera_inspector.default_data = data.camera_data;
    data.camera_inspector.default_settings = data.camera_settings;
    data
  }
}
impl Data {
  pub fn set_camera_inspector_defaults(&mut self) {
    let default = Self::default();
    self.camera_inspector.default_data = default.camera_data;
    self.camera_inspector.default_settings = default.camera_settings;
  }
}

pub struct Input {
  camera: CameraControllerInput,
}

impl app::Application for Cubes {
  type Data = Data;
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, mut data: Self::Data) -> Self {
    data.set_camera_inspector_defaults();

    let camera = Camera::new(&mut data.camera_data, &data.camera_settings, viewport.physical);

    let mut rng = SmallRng::seed_from_u64(101702198783735);

    let uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Cubes uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_camera(&camera)]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::VERTEX);

    let instance_buffer = {
      let buffer = BufferBuilder::default()
        .size((MAX_INSTANCES * size_of::<Instance>()) as BufferAddress)
        .storage_usage()
        .mapped_at_creation(true)
        .label("Cubes instance storage buffer")
        .build(&gfx.device);
      {
        let mut view = buffer.slice(..).get_mapped_range_mut();
        let instance_slice: &mut [Instance] = bytemuck::cast_slice_mut(&mut view);
        (0..data.num_cubes_to_generate)
          .map(|_| Instance::from_random_range(&mut rng, -data.cube_position_range..data.cube_position_range))
          .zip(instance_slice)
          .for_each(|(instance, place)| *place = instance);
      }
      buffer.unmap();
      buffer
    };
    let instance_binding = instance_buffer.binding(1, ShaderStages::VERTEX);

    let static_bind_group = CombinedBindGroupBuilder::default()
      .layout_entries(&[uniform_binding.layout, instance_binding.layout])
      .entries(&[uniform_binding.entry, instance_binding.entry])
      .layout_label("Cubes static bind group layout")
      .label("Cubes static bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));

    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Cubes pipeline layout")
      .bind_group_layouts(&[&static_bind_group.layout])
      .label("Cubes render pipeline")
      .vertex_module(&vertex_shader_module)
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);

    let index_buffer = {
      let data: Vec<_> = (0..MAX_INDICES).map(|i| {
        let cube = i / NUM_CUBE_INDICES;
        let cube_local = i % NUM_CUBE_INDICES;
        CUBE_INDICES[cube_local] + cube as u32 * NUM_CUBE_VERTICES as u32
      }).collect();
      BufferBuilder::default()
        .static_index_usage()
        .label("Cubes static index buffer")
        .build_with_data(&gfx.device, &data)
    };

    Self {
      data,

      camera,

      uniform_buffer,
      instance_buffer,
      static_bind_group,

      render_pipeline,

      index_buffer,

      rng,

      num_cubes: data.num_cubes_to_generate,
    }
  }
  fn into_data(self) -> Self::Data {
    self.data
  }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    self.camera.set_viewport(viewport.physical);
  }

  type Input = Input;
  fn process_input(&mut self, input: RawInput) -> Self::Input {
    let camera = CameraControllerInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.data.camera_inspector.add_to_menu(ui);
  }

  fn render(&mut self, RenderInput { gfx, frame, input, gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.data.camera_inspector.show_window_single(&gui, &mut self.camera, &mut self.data.camera_data, &mut self.data.camera_settings);
    self.camera.update(&mut self.data.camera_data, &self.data.camera_settings, &input.camera, frame.duration);
    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform::from_camera(&self.camera)]);

    gui.window("Cubes").show(&gui, |ui| {
      ui.grid("Grid", |ui| {
        ui.label("Cube instances");
        ui.add(DragValue::new(&mut self.data.num_cubes_to_generate).prefix("# ").speed(1000).clamp_range(0..=MAX_INSTANCES));
        ui.end_row();
        ui.label("Position range");
        ui.add(DragValue::new(&mut self.data.cube_position_range).speed(10).clamp_range(100..=1_000_000));
        ui.end_row();
      });
      if ui.button("Regenerate").clicked() {
        let instances: Vec<_> = (0..self.data.num_cubes_to_generate)
          .map(|_| Instance::from_random_range(&mut self.rng, -self.data.cube_position_range..self.data.cube_position_range))
          .collect();
        self.instance_buffer.write_all_data(&gfx.queue, &instances);
        self.num_cubes = self.data.num_cubes_to_generate
      }
    });

    let mut pass = gfx_frame.render_pass_builder().label("Cubes render pass").begin();
    pass.push_debug_group("Draw cubes");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.static_bind_group.entry, &[]);
    pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    let num_indices = self.num_cubes * NUM_CUBE_INDICES as u32;
    pass.draw_indexed(0..num_indices, 0, 0..1);
    pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Cubes")
    .run::<Cubes>()
    .unwrap();
}
