#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::{CommandBuffer, Features, PowerPreference};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx};
use gfx::camera::{Camera, CameraDebugging, CameraInput};
use gfx::debug_renderer::DebugRenderer;
use voxel::chunk::ChunkSize16;
use voxel::lod::render::{LodRenderData, LodRenderDataManager};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::settings::Settings;
use crate::stars::StarsRenderer;

pub mod settings;
pub mod stars;

pub struct VoxelPlanets {
  camera: Camera,
  camera_debugging: CameraDebugging,
  debug_renderer: DebugRenderer,

  camera_uniform: CameraUniform,
  settings: Settings,

  stars_renderer: StarsRenderer,
  voxel_renderer: VoxelRenderer,

  lod_render_data_manager: Box<dyn LodRenderDataManager<ChunkSize16>>,
  lod_render_data: LodRenderData,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

impl app::Application for VoxelPlanets {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;

    let mut camera = Camera::with_defaults_arcball_perspective(viewport);
    let extends = 4096.0 / 2.0;
    camera.arcball.distance = -extends * 2.0;
    camera.arcball.mouse_scroll_distance_speed = 1000.0;
    camera.far = 10000.0;
    let camera_debugging = CameraDebugging::default();
    let mut debug_renderer = DebugRenderer::new(gfx, camera.get_view_projection_matrix());

    let camera_uniform = CameraUniform::from_camera_sys(&camera);
    let mut settings = Settings::default();
    settings.light.rotation_y_degree = 270.0;
    settings.lod_octmap_transform = Isometry3::new(Vec3::new(-extends, -extends, -extends), Rotor3::identity());
    settings.lod_octmap_settings.lod_factor = 1.0;
    //settings.octree_settings.thread_pool_threads = 1;
    settings.lod_render_data_settings.debug_render_octree_nodes = true;
    settings.lod_render_data_settings.debug_render_octree_node_color = Vec4::new(0.0, 0.1, 0.0, 0.1);
    settings.lod_render_data_settings.debug_render_octree_node_empty_color = Vec4::new(0.1, 0.0, 0.0, 0.1);
    settings.auto_update = true;

    let stars_renderer = StarsRenderer::new(gfx, &camera);
    let voxel_renderer = VoxelRenderer::new(
      gfx,
      camera_uniform,
      settings.light.uniform,
      ModelUniform::identity(),
      None,
    );

    let mut lod_render_data_manager = settings.create_lod_render_data_manager();
    let lod_render_data = lod_render_data_manager.update(camera.get_position(), &settings.lod_render_data_settings, &mut debug_renderer, &gfx.device);

    Self {
      camera,
      camera_debugging,
      debug_renderer,

      camera_uniform,
      settings,

      stars_renderer,
      voxel_renderer,

      lod_render_data_manager,
      lod_render_data,
    }
  }


  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, screen_size: ScreenSize) {
    self.camera.viewport = screen_size.physical;
    self.stars_renderer.screen_resize(screen_size);
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
    self.camera_uniform.update_from_camera_sys(&self.camera);

    egui::Window::new("Voxel Planets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::ZERO)
      .show(&gui_frame, |ui| {
        self.settings.draw_light_gui(ui);
        let mut recreate_lod_render_data_manager = false;
        recreate_lod_render_data_manager |= self.settings.draw_volume_gui(ui);
        recreate_lod_render_data_manager |= self.settings.draw_extractor_gui(ui);
        if recreate_lod_render_data_manager {
          self.lod_render_data_manager = self.settings.create_lod_render_data_manager();
        }
        self.settings.draw_lod_chunk_vertices_manager_gui(ui, self.lod_render_data_manager.get_mesh_manager_parameters_mut());
        if self.settings.draw_lod_render_data_manager_gui(ui) { // Update is pressed or auto update is true
          self.debug_renderer.clear();
          self.lod_render_data = self.lod_render_data_manager.update(self.camera.get_position(), &self.settings.lod_render_data_settings, &mut self.debug_renderer, &gfx.device);
        }
        self.settings.draw_lod_render_data_gui(ui, &self.lod_render_data);
        self.settings.draw_stars_renderer_settings(ui);
      });

    // Render stars
    self.stars_renderer.render(gfx, &mut frame, &self.camera, &self.settings.stars_renderer_settings);

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

    // Debug render
    self.debug_renderer.render(gfx, &mut frame, self.camera.get_view_projection_matrix() * model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<VoxelPlanets>(Options {
    name: "Voxel Planets".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    ..Options::default()
  }).unwrap();
}
