#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use egui::{Align2, Ui};
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::{CommandBuffer, Features};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx};
use gfx::camera::{Camera, CameraInput};
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use gfx::texture::{GfxTexture, TextureBuilder};
use voxel::chunk::{ChunkVertices, GenericChunkSize};
use voxel::render::VoxelRenderer;
use voxel::uniform::{CameraUniform, LightSettings, ModelUniform};

use crate::marching_cubes_debugging::MarchingCubesDebugging;

mod marching_cubes_debugging;

pub struct TransvoxelDemo {
  camera: Camera,
  debug_renderer: DebugRenderer,

  camera_uniform: CameraUniform,
  light_settings: LightSettings,
  model_uniform: ModelUniform,

  depth_texture: GfxTexture,
  multisampled_framebuffer: GfxTexture,

  voxel_renderer: VoxelRenderer,

  marching_cubes_debugging: MarchingCubesDebugging,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

pub type C1 = GenericChunkSize<1>;

const MULTISAMPLE_COUNT: u32 = 4;
const EXTENDS: f32 = 0.5;

impl app::Application for TransvoxelDemo {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;

    let mut camera = Camera::with_defaults_arcball_orthographic(viewport);
    camera.arcball.distance = -2.0;
    let debug_renderer = DebugRenderer::new(gfx, MULTISAMPLE_COUNT, camera.get_view_projection_matrix());

    let camera_uniform = CameraUniform::from_camera_sys(&camera);
    let mut light_settings = LightSettings::default();
    light_settings.uniform.ambient = 0.2;
    light_settings.uniform.color = Vec3::new(0.0, 0.5, 0.35);
    let transform = Isometry3::new(Vec3::broadcast(-EXTENDS), Rotor3::identity());
    let model_uniform = ModelUniform::from_transform(transform);

    let depth_texture = TextureBuilder::new_depth_32_float(viewport)
      .with_sample_count(MULTISAMPLE_COUNT)
      .build(&gfx.device);
    let multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.surface, MULTISAMPLE_COUNT)
      .with_texture_label("Multisampling texture")
      .with_texture_view_label("Multisampling texture view")
      .build(&gfx.device);

    let voxel_renderer = VoxelRenderer::new(
      &gfx.device,
      &gfx.surface,
      camera_uniform,
      light_settings.uniform,
      model_uniform,
      MULTISAMPLE_COUNT,
      None,
      depth_texture.format,
    );

    let marching_cubes_debugging = MarchingCubesDebugging::default();

    Self {
      camera,
      debug_renderer,

      camera_uniform,
      light_settings,
      model_uniform,

      depth_texture,
      multisampled_framebuffer,

      voxel_renderer,

      marching_cubes_debugging,
    }
  }

  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport)
      .with_sample_count(MULTISAMPLE_COUNT)
      .build(&gfx.device);
    self.multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.surface, MULTISAMPLE_COUNT)
      .build(&gfx.device);
  }

  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Self::Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.camera.update(&input.camera, frame.time.delta, &gui_frame);
    self.camera_uniform.update_from_camera_sys(&self.camera);

    // Debug GUI
    self.marching_cubes_debugging.show_gui_window(gui_frame);
    egui::Window::new("Demo")
      .anchor(Align2::LEFT_BOTTOM, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        self.light_settings.render_gui(ui);
      });


    // Write uniforms, run MC to create vertices from voxels, and render them.
    self.voxel_renderer.update_camera_uniform(&gfx.queue, self.camera_uniform);
    self.voxel_renderer.update_light_uniform(&gfx.queue, self.light_settings.uniform);
    let mut chunk_vertices = ChunkVertices::new();
    self.marching_cubes_debugging.extract_chunk(&mut chunk_vertices);
    self.voxel_renderer.render_chunk_vertices(
      &gfx.device,
      &self.depth_texture.view,
      &mut frame.encoder,
      Some(&self.multisampled_framebuffer.view),
      &frame.output_texture,
      &chunk_vertices,
    );

    // Debug rendering.
    self.debug_renderer.clear();
    self.debug_renderer.draw_axes_lines(Vec3::broadcast(EXTENDS), EXTENDS);
    self.marching_cubes_debugging.debug_draw(&mut self.debug_renderer);
    self.debug_renderer.draw_triangle_vertices_wireframe_indexed(
      chunk_vertices.vertices().into_iter().map(|v| RegularVertex::new(v.position, Vec4::one())),
      chunk_vertices.indices().into_iter().copied(),
    );
    self.debug_renderer.draw_point_vertices(chunk_vertices.vertices().into_iter().map(|v| PointVertex::new(v.position, Vec4::one(), 10.0)));
    self.debug_renderer.render(gfx, &mut frame, Some(&self.multisampled_framebuffer), self.camera.get_view_projection_matrix() * self.model_uniform.model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<TransvoxelDemo>(Options {
    name: "Transvoxel".to_string(),
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    ..Options::default()
  }).unwrap();
}
