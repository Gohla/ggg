use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3};
use wgpu::CommandBuffer;

use app::{AppRunner, GuiFrame};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx};
use gfx::camera::{Camera, CameraInput};
use gfx::debug_renderer::DebugRenderer;
use os::Os;
use voxel::chunk::size::ChunkSize16;
use voxel::lod::render::{LodRenderData, LodRenderDataManager};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::settings::Settings;
use crate::stars::StarsRenderer;

pub mod settings;
pub mod stars;

pub struct VoxelPlanets {
  settings: Settings,

  cameras: Vec<Camera>,
  camera_uniform: CameraUniform,

  stars_renderer: StarsRenderer,
  voxel_renderer: VoxelRenderer,

  lod_octmap_transform: Isometry3,
  lod_render_data_manager: Box<dyn LodRenderDataManager<ChunkSize16>>,
  lod_render_data: LodRenderData,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

const EXTENDS: f32 = 4096.0 / 2.0;

impl app::Application for VoxelPlanets {
  type Config = Settings;

  #[profiling::function]
  fn new(os: &Os, gfx: &Gfx, mut settings: Self::Config) -> Self {
    settings.update_default_camera_settings();
    let lod_octmap_transform = Isometry3::new(Vec3::new(-EXTENDS, -EXTENDS, -EXTENDS), Rotor3::identity());

    let viewport = os.window.inner_size().physical;
    let camera_0 = Camera::new(viewport, &mut settings.camera_settings[0]);
    let camera_1 = Camera::new(viewport, &mut settings.camera_settings[1]);
    let cameras = vec![camera_0, camera_1];
    let selected_camera = settings.camera_debugging.get_selected_camera(&cameras);
    let camera_uniform = CameraUniform::from_camera(selected_camera);

    let stars_renderer = StarsRenderer::new(gfx, selected_camera.get_view_inverse_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      settings.light.uniform,
      ModelUniform::identity(),
      None,
    );

    let mut lod_render_data_manager = settings.create_lod_render_data_manager(gfx, lod_octmap_transform, selected_camera.get_view_projection_matrix());
    let lod_render_data = lod_render_data_manager.update(selected_camera.get_position(), &settings.lod_render_data_settings, &gfx.device);

    Self {
      settings,

      cameras,
      camera_uniform,

      stars_renderer,
      voxel_renderer,

      lod_octmap_transform,
      lod_render_data_manager,
      lod_render_data,
    }
  }

  fn into_config(self) -> Self::Config { self.settings }


  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, screen_size: ScreenSize) {
    for camera in &mut self.cameras {
      camera.set_viewport(screen_size.physical);
    }
    self.stars_renderer.screen_resize(screen_size);
  }


  type Input = Input;

  #[profiling::function]
  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.settings.camera_debugging.add_to_menu(ui);
  }


  #[profiling::function]
  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.settings.camera_debugging.show_debugging_gui_window_multiple_cameras(&gui_frame, &self.cameras, &mut self.settings.camera_settings);
    let (camera, camera_settings) = self.settings.camera_debugging.get_selected_camera_and_settings(&mut self.cameras, &mut self.settings.camera_settings);
    camera.update(camera_settings, &input.camera, frame.time.delta);
    self.camera_uniform.update_from_camera(camera);
    let camera_view_projection = camera.get_view_projection_matrix();
    let camera_direction_inverse = camera.get_direction_inverse();
    let camera_view_inverse_matrix = camera.get_view_inverse_matrix();

    let (recreate_lod_render_data_manager, update_lod_render_data) = egui::Window::new("Voxel Planets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::ZERO)
      .auto_sized()
      .show(&gui_frame, |ui| {
        let mut recreate = false;
        recreate |= self.settings.draw_reset_to_defaults_button(ui);
        self.settings.draw_light_gui(ui, camera_direction_inverse);
        recreate |= self.settings.draw_volume_gui(ui);
        recreate |= self.settings.draw_extractor_gui(ui);
        recreate |= self.settings.draw_lod_octmap_gui(ui);
        self.settings.draw_lod_chunk_mesh_manager_gui(ui, self.lod_render_data_manager.get_mesh_manager_parameters_mut());
        let update = self.settings.draw_lod_render_data_manager_gui(ui);
        self.settings.draw_lod_render_data_gui(ui, &self.lod_render_data);
        self.settings.draw_stars_renderer_settings(ui);
        (recreate, update)
      }).map_or((false, false), |r| r.inner.unwrap_or_default());

    // LOD render data
    if recreate_lod_render_data_manager {
      self.lod_render_data_manager = self.settings.create_lod_render_data_manager(gfx, self.lod_octmap_transform, self.cameras[0].get_view_projection_matrix());
    }
    if update_lod_render_data {
      self.lod_render_data = self.lod_render_data_manager.update(self.cameras[0].get_position(), &self.settings.lod_render_data_settings, &gfx.device);
    }

    // Render stars
    self.stars_renderer.render(gfx, &mut frame, camera_view_inverse_matrix, &self.settings.stars_renderer_settings);

    // Render voxels
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.settings.light.uniform);
    let model = self.lod_render_data.model;
    self.voxel_renderer.update_model_uniform(&gfx.queue, ModelUniform::new(model));
    self.voxel_renderer.render_lod_mesh(
      gfx,
      &mut frame,
      false,
      &self.lod_render_data,
    );

    // LOD render data debug draw (last so it draws over everything)
    self.lod_render_data_manager.debug_render(gfx, &mut frame, camera_view_projection, &self.lod_render_data);

    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Voxel Planets")
    .with_high_power_graphics_adapter()
    .request_graphics_device_features(DebugRenderer::request_features())
    .run::<VoxelPlanets>()
    .unwrap();
}
