use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::{CommandBuffer, Features};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx};
use gfx::camera::{Camera, CameraDebugging, CameraInput, CameraSettings};
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use voxel::chunk::{ChunkMesh, ChunkSize, ChunkSize2};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, LightSettings, ModelUniform};

use crate::surface_nets_debugging::SurfaceNetsDebugging;

mod surface_nets_debugging;

pub struct SurfaceNetsDemo {
  camera_settings: CameraSettings,
  camera_debugging: CameraDebugging,
  light_settings: LightSettings,

  camera: Camera,
  camera_uniform: CameraUniform,

  model_uniform: ModelUniform,

  debug_renderer: DebugRenderer,
  voxel_renderer: VoxelRenderer,

  surface_nets_debugging: SurfaceNetsDebugging,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

pub type C = ChunkSize2;

const EXTENDS: f32 = C::CELLS_IN_CHUNK_ROW_F32 / 2.0;

impl app::Application for SurfaceNetsDemo {
  type Config = ();

  fn new(os: &Os, gfx: &Gfx, _config: Self::Config) -> Self {
    let camera_settings = CameraSettings::with_defaults_arcball_orthographic();
    let camera_debugging = CameraDebugging::with_default_settings(camera_settings);
    let mut light_settings = LightSettings::default();
    light_settings.uniform.ambient = 0.2;
    light_settings.uniform.color = Vec3::new(0.0, 0.5, 0.35);

    let camera = Camera::new(os.window.get_inner_size().physical);
    let camera_uniform = CameraUniform::from_camera(&camera);

    let transform = Isometry3::new(Vec3::broadcast(-EXTENDS), Rotor3::identity());
    let model_uniform = ModelUniform::from_transform(transform);

    let debug_renderer = DebugRenderer::new(gfx, camera.get_view_projection_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      light_settings.uniform,
      model_uniform,
      None,
    );

    let surface_nets_debugging = SurfaceNetsDebugging::default();

    Self {
      camera_settings,
      camera_debugging,
      light_settings,

      camera,
      camera_uniform,

      model_uniform,

      debug_renderer,
      voxel_renderer,

      surface_nets_debugging,
    }
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
    self.camera_debugging.add_to_menu(ui);
  }


  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Self::Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.camera_debugging.show_debugging_gui_window(&gui_frame, &self.camera, &mut self.camera_settings);
    self.camera.update(&mut self.camera_settings, &input.camera, frame.time.delta);
    self.camera_uniform.update_from_camera(&self.camera);

    // Debug GUI
    self.surface_nets_debugging.show_gui_window(gui_frame);
    egui::Window::new("Demo")
      .anchor(Align2::LEFT_BOTTOM, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        self.light_settings.render_gui(ui);
      });

    // Write uniforms, run SN to create vertices from voxels, and render them.
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.light_settings.uniform);
    let mut chunk_vertices = ChunkMesh::new();
    self.surface_nets_debugging.extract_chunk(&mut chunk_vertices);
    self.voxel_renderer.render_chunk_vertices(
      gfx,
      &mut frame,
      true,
      &chunk_vertices,
    );

    // Debug rendering.
    self.debug_renderer.clear();
    self.debug_renderer.draw_axes_lines(Vec3::broadcast(EXTENDS), EXTENDS);
    self.surface_nets_debugging.debug_draw(&mut self.debug_renderer);
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
  app::run::<SurfaceNetsDemo>(Options {
    name: "Transvoxel".to_string(),
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    sample_count: 4,
    ..Options::default()
  }).unwrap();
}
