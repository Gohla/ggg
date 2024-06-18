use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3};
use wgpu::CommandBuffer;
use wgpu::util::StagingBelt;

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::camera::Camera;
use gfx::camera::controller::CameraControllerInput;
use gfx::debug_renderer::DebugRenderer;
use gfx::Gfx;
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
  camera_controller: CameraControllerInput,
}

const EXTENDS: f32 = 4096.0 / 2.0;

impl app::Application for VoxelPlanets {
  type Data = Settings;

  #[profiling::function]
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, mut settings: Self::Data) -> Self {
    settings.update_default_camera_settings();
    let lod_octmap_transform = Isometry3::new(Vec3::new(-EXTENDS, -EXTENDS, -EXTENDS), Rotor3::identity());

    let camera_0 = Camera::new(&mut settings.camera_data[0], &mut settings.camera_settings[0], viewport.physical);
    let camera_1 = Camera::new(&mut settings.camera_data[1], &mut settings.camera_settings[1], viewport.physical);
    let cameras = vec![camera_0, camera_1];
    let selected_camera = settings.camera_debugging.selected_camera(&cameras);
    let camera_uniform = CameraUniform::from_camera(selected_camera);

    let stars_renderer = StarsRenderer::new(gfx, *selected_camera.inverse_view_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      settings.light.uniform,
      ModelUniform::identity(),
      None,
      StagingBelt::new(4096 * 1024), // 4 MiB staging belt
    );

    let lod_render_data_manager = settings.create_lod_render_data_manager(gfx, lod_octmap_transform, *selected_camera.view_projection_matrix());

    Self {
      settings,

      cameras,
      camera_uniform,

      stars_renderer,
      voxel_renderer,

      lod_octmap_transform,
      lod_render_data_manager,
      lod_render_data: LodRenderData::default(),
    }
  }
  fn into_data(self) -> Self::Data { self.settings }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    for camera in &mut self.cameras {
      camera.set_viewport(viewport.physical);
    }
    self.stars_renderer.resize_viewport(viewport);
  }

  type Input = Input;
  #[profiling::function]
  fn process_input(&mut self, input: RawInput) -> Input {
    let camera_controller = CameraControllerInput::from(&input);
    Input { camera_controller }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.settings.camera_debugging.add_to_menu(ui);
  }

  #[profiling::function]
  fn render(&mut self, RenderInput { gfx, frame, input, mut gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.settings.camera_debugging.show(&gui, &mut self.cameras, &mut self.settings.camera_data, &mut self.settings.camera_settings);
    let (camera, camera_data, camera_settings) =
      self.settings.camera_debugging.selected(&mut self.cameras, &mut self.settings.camera_data, &self.settings.camera_settings);
    camera.update(camera_data, camera_settings, &input.camera_controller, frame.duration);
    self.camera_uniform.update_from_camera(camera);
    let camera_view_projection = *camera.view_projection_matrix();
    let camera_inverse_direction = camera.inverse_direction_vector();
    let camera_view_inverse_matrix = *camera.inverse_view_matrix();

    let (recreate_lod_render_data_manager, update_lod_render_data) = gui.window("Voxel Planets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::ZERO)
      .auto_sized()
      .show(&gui, |ui| {
        let mut recreate = false;
        recreate |= self.settings.draw_reset_to_defaults_button(ui);
        self.settings.draw_light_gui(ui, camera_inverse_direction);
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
      self.lod_render_data_manager = self.settings.create_lod_render_data_manager(gfx, self.lod_octmap_transform, *self.cameras[0].view_projection_matrix());
    }
    if update_lod_render_data {
      self.lod_render_data_manager.update(self.cameras[0].position(), &self.settings.lod_render_data_settings, &mut self.lod_render_data);
    }

    // Render stars
    self.stars_renderer.render(gfx, &mut gfx_frame, camera_view_inverse_matrix, &self.settings.stars_renderer_settings);

    // Render voxels
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.settings.light.uniform);
    let model = self.lod_render_data.model;
    self.voxel_renderer.update_model_uniform(&gfx.queue, ModelUniform::new(model));
    self.voxel_renderer.render_lod_mesh(gfx, &mut gfx_frame, false, &self.lod_render_data);

    // LOD render data debug draw (last so it draws over everything)
    self.lod_render_data_manager.debug_render(gfx, &mut gfx_frame, camera_view_projection, &self.lod_render_data);

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

#[global_allocator]
static GLOBAL: os::profile::Allocator = os::profile::create_allocator();
