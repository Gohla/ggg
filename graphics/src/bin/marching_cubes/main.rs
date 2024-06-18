use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::CommandBuffer;
use wgpu::util::StagingBelt;

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::camera::controller::CameraControllerInput;
use gfx::camera::projection::ProjectionType;
use gfx::camera::system::{CameraData, CameraSystem};
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use gfx::Gfx;
use os::Os;
use voxel::chunk::mesh::ChunkMesh;
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::data::Data;

mod data;
mod inspector;

pub struct MarchingCubesDemo {
  data: Data,

  camera_system: CameraSystem,

  camera_uniform: CameraUniform,
  model_uniform: ModelUniform,

  debug_renderer: DebugRenderer,
  voxel_renderer: VoxelRenderer,
}

pub struct Input {
  camera: CameraControllerInput,
}

/// Extends of the cube to render the result of marching cubes in.
const EXTENDS: f32 = 0.5;

impl app::Application for MarchingCubesDemo {
  type Data = Data;
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, mut data: Self::Data) -> Self {
    let mut camera_system = {
      let mut default = CameraData::default();
      default.state.controller.arcball.distance = 2.0;
      default.settings.projection.projection_type = ProjectionType::Orthographic;
      data.camera_manager_state.take_into(default, viewport.physical)
    };
    let camera = camera_system.active_camera();

    let camera_uniform = CameraUniform::from_camera(&camera);
    let transform = Isometry3::new(Vec3::broadcast(-EXTENDS), Rotor3::identity());
    let model_uniform = ModelUniform::from_transform(transform);

    let debug_renderer = DebugRenderer::new(gfx, *camera.view_projection_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      data.light.uniform,
      model_uniform,
      None,
      StagingBelt::new(256), // Tiny staging belt: tiny buffers in this demo.
    );

    Self {
      data,

      camera_system,

      camera_uniform,
      model_uniform,

      debug_renderer,
      voxel_renderer,
    }
  }
  fn into_data(mut self) -> Self::Data {
    self.data.camera_manager_state = self.camera_system.into();
    self.data
  }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    self.camera_system.set_viewport(viewport.physical);
  }

  type Input = Input;
  fn process_input(&mut self, input: RawInput) -> Self::Input {
    let camera = CameraControllerInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.data.camera_inspector.add_to_menu(ui);
  }

  fn render(&mut self, RenderInput { gfx, frame, input, mut gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.data.camera_inspector.show_window(&gui, &mut self.camera_system);
    let mut camera = self.camera_system.active_camera();
    camera.update(&input.camera, frame.duration);
    self.camera_uniform.update_from_camera(&camera);

    // Debug GUI
    self.data.inspector.show_window(&gui);
    gui.window("Demo").anchor(Align2::LEFT_BOTTOM, egui::Vec2::default()).show(&gui, |ui| {
      self.data.light.show(ui, camera.inverse_direction_vector());
    });


    // Write uniforms, run MC to create vertices from voxels, and render them.
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.data.light.uniform);
    let mut chunk_vertices = ChunkMesh::new();
    self.data.inspector.extract_chunk(&mut chunk_vertices);
    self.voxel_renderer.render_chunk_vertices(gfx, &mut gfx_frame, true, &chunk_vertices);

    // Debug rendering.
    self.debug_renderer.clear();
    self.debug_renderer.draw_axes_lines(Vec3::broadcast(EXTENDS), EXTENDS);
    self.data.inspector.debug_draw(&mut self.debug_renderer);
    self.debug_renderer.draw_triangle_vertices_wireframe_indexed(
      chunk_vertices.vertices().into_iter().map(|v| RegularVertex::new(v.position, Vec4::one())),
      chunk_vertices.indices().into_iter().map(|i| *i as u32),
    );
    self.debug_renderer.draw_point_vertices(chunk_vertices.vertices().into_iter().map(|v| PointVertex::new(v.position, Vec4::one(), 10.0)));
    self.debug_renderer.render(gfx, &mut gfx_frame, *camera.view_projection_matrix() * self.model_uniform.model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Marching Cubes")
    .request_graphics_device_features(DebugRenderer::request_features())
    .with_sample_count(4)
    .run::<MarchingCubesDemo>()
    .unwrap();
}
