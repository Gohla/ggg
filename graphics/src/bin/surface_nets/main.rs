use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::CommandBuffer;
use wgpu::util::StagingBelt;

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::camera::Camera;
use gfx::camera::controller::CameraControllerInput;
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use gfx::Gfx;
use os::Os;
use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::size::{ChunkSize, ChunkSize2, ChunkSize6};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::data::Data;
use crate::inspector::SurfaceNetsInspector;

mod inspector;
mod data;

pub struct SurfaceNetsDemo {
  data: Data,

  camera: Camera,
  camera_uniform: CameraUniform,

  model_uniform: ModelUniform,

  debug_renderer: DebugRenderer,
  voxel_renderer: VoxelRenderer,
}

#[derive(Default)]
pub struct Input {
  camera: CameraControllerInput,
}

pub type C2 = ChunkSize2;
pub type C6 = ChunkSize6;

const EXTENDS: f32 = C2::CELLS_IN_CHUNK_ROW_F32 / 2.0;

impl app::Application for SurfaceNetsDemo {
  type Data = Data;
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, mut data: Self::Data) -> Self {
    data.set_camera_inspector_defaults();

    let camera = Camera::new(&mut data.camera_data, &data.camera_settings, viewport.physical);
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
      StagingBelt::new(1024), // Small staging belt: small buffers in this demo.
    );

    Self {
      data,

      camera,
      camera_uniform,

      model_uniform,

      debug_renderer,
      voxel_renderer,
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

  fn render(&mut self, RenderInput { gfx, frame, input, mut gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.data.camera_inspector.show_window_single(&gui, &mut self.camera, &mut self.data.camera_data, &mut self.data.camera_settings);
    self.camera.update(&mut self.data.camera_data, &self.data.camera_settings, &input.camera, frame.duration);
    self.camera_uniform.update_from_camera(&self.camera);

    // Debug GUI
    self.data.surface_nets.show_window(&gui);
    gui.window("Demo").anchor(Align2::LEFT_BOTTOM, egui::Vec2::default()).show(&gui, |ui| {
      self.data.light.show(ui, self.camera.inverse_direction_vector());
    });

    // Write uniforms
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.data.light.uniform);

    // Extract mesh using surface nets and debug draw.
    self.debug_renderer.clear();
    let mut chunk_vertices = ChunkMesh::new();
    self.data.surface_nets.extract_chunk_and_debug_draw(&mut chunk_vertices, &mut self.debug_renderer);
    self.voxel_renderer.render_chunk_vertices(gfx, &mut gfx_frame, true, &chunk_vertices);

    // Debug rendering.
    self.debug_renderer.draw_axes_lines(Vec3::broadcast(EXTENDS), EXTENDS);
    self.debug_renderer.draw_triangle_vertices_wireframe_indexed(
      chunk_vertices.vertices().into_iter().map(|v| RegularVertex::new(v.position, Vec4::one())),
      chunk_vertices.indices().into_iter().map(|i| *i as u32),
    );
    self.debug_renderer.draw_point_vertices(chunk_vertices.vertices().into_iter().map(|v| PointVertex::new(v.position, Vec4::one(), 10.0)));
    self.debug_renderer.render(gfx, &mut gfx_frame, *self.camera.view_projection_matrix() * self.model_uniform.model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Surface Nets")
    .request_graphics_device_features(DebugRenderer::request_features())
    .with_sample_count(4)
    .run::<SurfaceNetsDemo>()
    .unwrap();
}
