use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::CommandBuffer;
use wgpu::util::StagingBelt;

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::camera::{Camera, CameraInput};
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use gfx::Gfx;
use os::Os;
use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::size::{ChunkSize, ChunkSize2, ChunkSize6};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::config::Config;
use crate::surface_nets_debugging::SurfaceNetsDebugging;

mod surface_nets_debugging;
mod config;

pub struct SurfaceNetsDemo {
  config: Config,

  camera: Camera,
  camera_uniform: CameraUniform,

  model_uniform: ModelUniform,

  debug_renderer: DebugRenderer,
  voxel_renderer: VoxelRenderer,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

pub type C2 = ChunkSize2;
pub type C6 = ChunkSize6;

const EXTENDS: f32 = C2::CELLS_IN_CHUNK_ROW_F32 / 2.0;

impl app::Application for SurfaceNetsDemo {
  type Config = Config;

  fn new(_os: &Os, gfx: &Gfx, screen_size: ScreenSize, mut config: Self::Config) -> Self {
    config.update_default_camera_settings();

    let camera = Camera::new(screen_size.physical, &mut config.camera_settings);
    let camera_uniform = CameraUniform::from_camera(&camera);

    let transform = Isometry3::new(Vec3::broadcast(-EXTENDS), Rotor3::identity());
    let model_uniform = ModelUniform::from_transform(transform);

    let debug_renderer = DebugRenderer::new(gfx, camera.get_view_projection_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      config.light_settings.uniform,
      model_uniform,
      None,
      StagingBelt::new(1024), // Small staging belt: small buffers in this demo.
    );

    Self {
      config,

      camera,
      camera_uniform,

      model_uniform,

      debug_renderer,
      voxel_renderer,
    }
  }

  fn into_config(self) -> Self::Config {
    self.config
  }


  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, screen_size: ScreenSize) {
    self.camera.set_viewport(screen_size.physical);
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.config.camera_debugging.add_to_menu(ui);
  }


  fn render<'a>(&mut self, RenderInput { gfx, cycle, mut frame, gui, input, .. }: RenderInput<'a, Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.config.camera_debugging.show_debugging_gui_window(gui, &self.camera, &mut self.config.camera_settings);
    self.camera.update(&mut self.config.camera_settings, &input.camera, cycle.duration);
    self.camera_uniform.update_from_camera(&self.camera);

    // Debug GUI
    self.config.surface_nets_debugging.show_gui_window(gui);
    egui::Window::new("Demo")
      .anchor(Align2::LEFT_BOTTOM, egui::Vec2::default())
      .show(gui, |ui| {
        self.config.light_settings.render_gui(ui, self.camera.get_direction_inverse());
      });

    // Write uniforms
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.config.light_settings.uniform);

    // Extract mesh using surface nets and debug draw.
    self.debug_renderer.clear();
    let mut chunk_vertices = ChunkMesh::new();
    self.config.surface_nets_debugging.extract_chunk_and_debug_draw(&mut chunk_vertices, &mut self.debug_renderer);
    self.voxel_renderer.render_chunk_vertices(
      gfx,
      &mut frame,
      true,
      &chunk_vertices,
    );

    // Debug rendering.
    self.debug_renderer.draw_axes_lines(Vec3::broadcast(EXTENDS), EXTENDS);
    self.debug_renderer.draw_triangle_vertices_wireframe_indexed(
      chunk_vertices.vertices().into_iter().map(|v| RegularVertex::new(v.position, Vec4::one())),
      chunk_vertices.indices().into_iter().map(|i| *i as u32),
    );
    self.debug_renderer.draw_point_vertices(chunk_vertices.vertices().into_iter().map(|v| PointVertex::new(v.position, Vec4::one(), 10.0)));
    self.debug_renderer.render(gfx, &mut frame, self.camera.get_view_projection_matrix() * self.model_uniform.model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Surface Nets")
    .with_high_power_graphics_adapter()
    .request_graphics_device_features(DebugRenderer::request_features())
    .with_sample_count(4)
    .run::<SurfaceNetsDemo>()
    .unwrap();
}
