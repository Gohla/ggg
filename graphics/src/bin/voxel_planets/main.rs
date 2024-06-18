use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3};
use wgpu::CommandBuffer;
use wgpu::util::StagingBelt;

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::camera::{CameraSettings, CameraState};
use gfx::camera::controller::{ArcballSettings, ArcballState, CameraControllerInput, CameraControllerSettings, CameraControllerState, ControlType};
use gfx::camera::projection::CameraProjectionSettings;
use gfx::camera::system::{CameraData, CameraSystem};
use gfx::debug_renderer::DebugRenderer;
use gfx::Gfx;
use os::Os;
use voxel::chunk::size::ChunkSize16;
use voxel::lod::render::{LodRenderData, LodRenderDataManager};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::data::Data;
use crate::stars::StarsRenderer;

pub mod data;
pub mod stars;

pub struct VoxelPlanets {
  data: Data,

  camera_system: CameraSystem,
  camera_uniform: CameraUniform,

  stars_renderer: StarsRenderer,
  voxel_renderer: VoxelRenderer,

  lod_octmap_transform: Isometry3,
  lod_render_data_manager: Box<dyn LodRenderDataManager<ChunkSize16>>,
  lod_render_data: LodRenderData,
}

pub struct Input {
  camera: CameraControllerInput,
}

const EXTENDS: f32 = 4096.0 / 2.0;

impl app::Application for VoxelPlanets {
  type Data = Data;
  #[profiling::function]
  fn new(_os: &Os, gfx: &Gfx, viewport: ScreenSize, mut data: Self::Data) -> Self {
    let lod_octmap_transform = Isometry3::new(Vec3::new(-EXTENDS, -EXTENDS, -EXTENDS), Rotor3::identity());

    let default_camera_data = CameraData {
      state: CameraState {
        controller: CameraControllerState {
          arcball: ArcballState {
            distance: EXTENDS * 2.0 - 1.0,
            ..Default::default()
          },
          ..Default::default()
        }
      },
      settings: CameraSettings {
        controller: CameraControllerSettings {
          control_type: ControlType::Arcball,
          arcball: ArcballSettings {
            mouse_movement_panning_speed: 2.0,
            keyboard_panning_speed: 1000.0,
            mouse_scroll_distance_speed: 100.0,
            ..Default::default()
          },
        },
        projection: CameraProjectionSettings {
          far: 10000.0,
          ..Default::default()
        },
      },
    };
    let mut camera_system = data.camera_manager_state.take_into(default_camera_data, viewport.physical);
    camera_system.ensure_minimum_camera_count(2, viewport.physical);
    let camera = camera_system.active_camera();
    let camera_uniform = CameraUniform::from_camera(camera.camera);

    let stars_renderer = StarsRenderer::new(gfx, *camera.inverse_view_matrix());
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      data.light.uniform,
      ModelUniform::identity(),
      None,
      StagingBelt::new(4096 * 1024), // 4 MiB staging belt
    );

    let lod_render_data_manager = data.create_lod_render_data_manager(gfx, lod_octmap_transform, *camera.view_projection_matrix());

    Self {
      data,

      camera_system,
      camera_uniform,

      stars_renderer,
      voxel_renderer,

      lod_octmap_transform,
      lod_render_data_manager,
      lod_render_data: LodRenderData::default(),
    }
  }
  fn into_data(mut self) -> Self::Data {
    self.data.camera_manager_state = self.camera_system.into();
    self.data
  }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    self.camera_system.set_viewport(viewport.physical);
    self.stars_renderer.resize_viewport(viewport);
  }

  type Input = Input;
  #[profiling::function]
  fn process_input(&mut self, input: RawInput) -> Input {
    let camera_controller = CameraControllerInput::from(&input);
    Input { camera: camera_controller }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.data.camera_inspector.add_to_menu(ui);
  }

  #[profiling::function]
  fn render(&mut self, RenderInput { gfx, frame, input, mut gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.data.camera_inspector.show_window(&gui, &mut self.camera_system);
    let mut camera = self.camera_system.active_camera();
    camera.update(&input.camera, frame.duration);
    self.camera_uniform.update_from_camera(&camera);
    let camera_view_projection = *camera.view_projection_matrix();
    let camera_inverse_direction = camera.inverse_direction_vector();
    let camera_view_inverse_matrix = *camera.inverse_view_matrix();

    let (recreate_lod_render_data_manager, update_lod_render_data) = gui.window("Voxel Planets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::ZERO)
      .auto_sized()
      .show(&gui, |ui| {
        let mut recreate = false;
        recreate |= self.data.draw_reset_to_defaults_button(ui);
        self.data.draw_light_gui(ui, camera_inverse_direction);
        recreate |= self.data.draw_volume_gui(ui);
        recreate |= self.data.draw_extractor_gui(ui);
        recreate |= self.data.draw_lod_octmap_gui(ui);
        self.data.draw_lod_chunk_mesh_manager_gui(ui, self.lod_render_data_manager.get_mesh_manager_parameters_mut());
        let update = self.data.draw_lod_render_data_manager_gui(ui);
        self.data.draw_lod_render_data_gui(ui, &self.lod_render_data);
        self.data.draw_stars_renderer_settings(ui);
        (recreate, update)
      }).map_or((false, false), |r| r.inner.unwrap_or_default());

    // LOD render data
    if recreate_lod_render_data_manager {
      self.lod_render_data_manager = self.data.create_lod_render_data_manager(gfx, self.lod_octmap_transform, *self.camera_system.camera_at(0).view_projection_matrix());
    }
    if update_lod_render_data {
      self.lod_render_data_manager.update(self.camera_system.camera_at(0).position(), &self.data.lod_render_data_settings, &mut self.lod_render_data);
    }

    // Render stars
    self.stars_renderer.render(gfx, &mut gfx_frame, camera_view_inverse_matrix, &self.data.stars_renderer_settings);

    // Render voxels
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.data.light.uniform);
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
