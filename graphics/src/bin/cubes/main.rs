///! Rendering a lot of generic cubes. Based on https://twitter.com/SebAaltonen/status/1315982782439591938.

use std::mem::size_of;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use egui::{DragValue, Ui};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{Backends, BindGroup, BufferAddress, CommandBuffer, IndexFormat, PowerPreference, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraDebugging, CameraInput};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gui_widget::UiWidgetsExt;

const NUM_CUBE_INDICES: usize = 3 * 3 * 2;
const NUM_CUBE_VERTICES: usize = 8;
const CUBE_INDICES: [u32; NUM_CUBE_INDICES] = [
  0, 2, 1, 2, 3, 1,
  5, 4, 1, 1, 4, 0,
  0, 4, 6, 0, 6, 2,
];

const MAX_INSTANCES: usize = 7_500_000;
const MAX_INDICES: usize = MAX_INSTANCES * NUM_CUBE_INDICES;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  camera_position: Vec4,
  view_projection: Mat4,
}

impl Uniform {
  pub fn from_camera_sys(camera_sys: &Camera) -> Self {
    Self {
      camera_position: camera_sys.get_position().into_homogeneous_point(),
      view_projection: camera_sys.get_view_projection_matrix(),
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
  camera: Camera,
  camera_debugging: CameraDebugging,

  uniform_buffer: GfxBuffer,
  instance_buffer: GfxBuffer,
  static_bind_group: BindGroup,

  render_pipeline: RenderPipeline,

  index_buffer: GfxBuffer,

  num_cubes_to_generate: u32,
  cube_position_range: f32,
  rng: SmallRng,

  num_cubes: u32,
}

pub struct Input {
  camera: CameraInput,
}

impl app::Application for Cubes {
  type Config = ();

  fn new(os: &Os, gfx: &Gfx, _config: Self::Config) -> Self {
    let viewport = os.window.get_inner_size().physical;
    let mut camera = Camera::with_defaults_arcball_perspective(viewport);
    camera.arcball.mouse_scroll_distance_speed = 100.0;
    let camera_debugging = CameraDebugging::default();

    let num_cubes_to_generate = 100_000;
    let cube_position_range = 1000.0;
    let mut rng = SmallRng::seed_from_u64(101702198783735);

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Cubes uniform buffer")
      .build_with_data(&gfx.device, &[Uniform::from_camera_sys(&camera)]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX);

    let instance_buffer = {
      let buffer = BufferBuilder::new()
        .with_size((MAX_INSTANCES * size_of::<Instance>()) as BufferAddress)
        .with_storage_usage()
        .with_mapped_at_creation(true)
        .with_label("Cubes instance storage buffer")
        .build(&gfx.device);
      {
        let mut view = buffer.slice(..).get_mapped_range_mut();
        let instance_slice: &mut [Instance] = bytemuck::cast_slice_mut(&mut view);
        (0..num_cubes_to_generate)
          .map(|_| Instance::from_random_range(&mut rng, -cube_position_range..cube_position_range))
          .zip(instance_slice)
          .for_each(|(instance, place)| *place = instance);
      }
      buffer.unmap();
      buffer
    };
    let (instance_bind_group_layout_entry, instance_bind_group_entry) = instance_buffer.create_storage_binding_entries(1, ShaderStages::VERTEX, true);

    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry, instance_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry, instance_bind_group_entry])
      .with_layout_label("Cubes static bind group layout")
      .with_label("Cubes static bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("frag"));

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&static_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_depth_texture(gfx.depth_stencil_format().unwrap())
      .with_layout_label("Cubes pipeline layout")
      .with_label("Cubes render pipeline")
      .build(&gfx.device);

    let index_buffer = {
      let data: Vec<_> = (0..MAX_INDICES).map(|i| {
        let cube = i / NUM_CUBE_INDICES;
        let cube_local = i % NUM_CUBE_INDICES;
        CUBE_INDICES[cube_local] + cube as u32 * NUM_CUBE_VERTICES as u32
      }).collect();
      BufferBuilder::new()
        .with_static_index_usage()
        .with_label("Cubes static index buffer")
        .build_with_data(&gfx.device, &data)
    };

    Self {
      camera,
      camera_debugging,

      uniform_buffer,
      instance_buffer,
      static_bind_group,

      render_pipeline,

      index_buffer,

      num_cubes_to_generate,
      cube_position_range,
      rng,

      num_cubes: num_cubes_to_generate,
    }
  }

  fn get_config(&self) -> &Self::Config { &() }


  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera.viewport = viewport;
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.camera_debugging.add_to_menu(ui);
  }


  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera.update(&input.camera, frame.time.delta);
    self.camera_debugging.show_debugging_gui_window(&gui_frame, &mut self.camera);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::from_camera_sys(&self.camera)]);

    egui::Window::new("Cubes").show(&gui_frame, |ui| {
      ui.grid("Grid", |ui| {
        ui.label("Cube instances");
        ui.add(DragValue::new(&mut self.num_cubes_to_generate).prefix("# ").speed(1000).clamp_range(0..=MAX_INSTANCES));
        ui.end_row();
        ui.label("Position range");
        ui.add(DragValue::new(&mut self.cube_position_range).speed(10).clamp_range(100..=1_000_000));
        ui.end_row();
      });
      if ui.button("Regenerate").clicked() {
        // OPTO: get mutable slice to buffer to prevent copy, but that requires use of the weird map_async API...
        let instances: Vec<_> = (0..self.num_cubes_to_generate)
          .map(|_| Instance::from_random_range(&mut self.rng, -self.cube_position_range..self.cube_position_range))
          .collect();
        self.instance_buffer.write_whole_data(&gfx.queue, &instances);
        self.num_cubes = self.num_cubes_to_generate
      }
    });

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Cubes render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, &mut frame, true);
    render_pass.push_debug_group("Draw cubes");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    let num_indices = self.num_cubes * NUM_CUBE_INDICES as u32;
    render_pass.draw_indexed(0..num_indices, 0, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<Cubes>(Options {
    name: "Cubes".to_string(),
    graphics_backends: Backends::all(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
