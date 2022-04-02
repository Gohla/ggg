#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use egui::Ui;
use ultraviolet::{Isometry3, Rotor3, Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, Features, IndexFormat, PowerPreference, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraInput};
use gfx::debug_renderer::DebugRenderer;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use voxel::chunk::Vertex;
use voxel::lod::chunk::LodChunkManager;
use voxel::lod::mesh::{LodMesh, LodMeshManager};
use voxel::uniform::{CameraUniform, ModelUniform};

use crate::settings::Settings;

pub mod settings;

pub struct VoxelMeshing {
  camera: Camera,
  debug_renderer: DebugRenderer,

  camera_uniform: CameraUniform,
  settings: Settings,

  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  model_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  depth_texture: GfxTexture,
  render_pipeline: RenderPipeline,

  lod_chunk_manager: Box<dyn LodChunkManager>,
  lod_mesh_manager: LodMeshManager,
  lod_mesh: LodMesh,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

impl app::Application for VoxelMeshing {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;

    let mut camera = Camera::with_defaults_arcball_perspective(viewport);
    let extends = 4096.0 / 2.0;
    camera.arcball.distance = -extends * 2.0;
    camera.arcball.mouse_scroll_distance_speed = 1000.0;
    camera.far = 10000.0;
    let mut debug_renderer = DebugRenderer::new(gfx, 1, camera.get_view_projection_matrix());

    let camera_uniform = CameraUniform::from_camera_sys(&camera);
    let mut settings = Settings::default();
    settings.light.rotation_y_degree = 270.0;
    settings.lod_octmap_transform = Isometry3::new(Vec3::new(-extends, -extends, -extends), Rotor3::identity());
    settings.lod_mesh_manager_settings.render_regular_chunks = true;
    settings.lod_mesh_manager_settings.render_transition_lo_x_chunks = false;
    settings.lod_mesh_manager_settings.render_transition_hi_x_chunks = false;
    settings.lod_mesh_manager_settings.render_transition_lo_y_chunks = false;
    settings.lod_mesh_manager_settings.render_transition_hi_y_chunks = false;
    settings.lod_mesh_manager_settings.render_transition_lo_z_chunks = false;
    settings.lod_mesh_manager_settings.render_transition_hi_z_chunks = false;
    settings.lod_mesh_manager_settings.debug_render_octree_nodes = true;
    settings.lod_mesh_manager_settings.debug_render_octree_node_color = Vec4::new(0.0, 0.1, 0.0, 0.1);
    settings.lod_mesh_manager_settings.debug_render_octree_node_empty_color = Vec4::new(0.1, 0.0, 0.0, 0.1);
    settings.lod_octmap_settings.lod_factor = 2.0;
    //settings.octree_settings.thread_pool_threads = 1;
    settings.auto_update = true;

    let depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&voxel::render::get_vertex_shader());
    let fragment_shader_module = gfx.device.create_shader_module(&voxel::render::get_fragment_shader());

    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .build_with_data(&gfx.device, &[camera_uniform]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(&gfx.device, &[settings.light.uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let model_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Model uniform buffer")
      .build_with_data(&gfx.device, &[ModelUniform::identity()]);
    let (model_uniform_bind_group_layout_entry, model_uniform_bind_group_entry) = model_uniform_buffer.create_uniform_binding_entries(2, ShaderStages::VERTEX);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry, model_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry, model_uniform_bind_group_entry])
      .with_layout_label("Voxel meshing uniform bind group layout")
      .with_label("Voxel meshing uniform bind group")
      .build(&gfx.device);

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      //.with_cull_mode(Some(Face::Back))
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Voxel meshing pipeline layout")
      .with_label("Voxel meshing render pipeline")
      .build(&gfx.device);

    let mut lod_chunk_manager = settings.create_lod_chunk_manager();
    let mut lod_mesh_manager = LodMeshManager::new();
    let lod_mesh = lod_mesh_manager.update(&mut lod_chunk_manager, camera.get_position(), &settings.lod_mesh_manager_settings, &mut debug_renderer, &gfx.device);

    Self {
      camera,
      debug_renderer,

      camera_uniform,
      settings,

      camera_uniform_buffer,
      light_uniform_buffer,
      model_uniform_buffer,
      uniform_bind_group,
      depth_texture,
      render_pipeline,

      lod_chunk_manager,
      lod_mesh_manager,
      lod_mesh,
    }
  }


  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera.update(&input.camera, frame.time.delta, &gui_frame);
    self.camera_uniform.update_from_camera_sys(&self.camera);

    egui::Window::new("Voxel Meshing").show(&gui_frame, |ui| {
      self.settings.draw_light_gui(ui);
      let mut recreate_lod_chunk_manager = false;
      recreate_lod_chunk_manager |= self.settings.draw_volume_gui(ui);
      recreate_lod_chunk_manager |= self.settings.draw_meshing_algorithm_gui(ui);
      if recreate_lod_chunk_manager {
        self.lod_chunk_manager = self.settings.create_lod_chunk_manager();
      }
      self.settings.draw_lod_chunk_manager_gui(ui, &mut self.lod_chunk_manager);
      if self.settings.draw_lod_mesh_manager_gui(ui) { // Update is pressed or auto update is true
        self.debug_renderer.clear();
        self.lod_mesh = self.lod_mesh_manager.update(&mut self.lod_chunk_manager, self.camera.get_position(), &self.settings.lod_mesh_manager_settings, &mut self.debug_renderer, &gfx.device);
      }
      self.settings.draw_lod_mesh_gui(ui, &self.lod_mesh);
    });

    self.camera_uniform_buffer.write_whole_data(&gfx.queue, &[self.camera_uniform]);
    self.light_uniform_buffer.write_whole_data(&gfx.queue, &[self.settings.light.uniform]);
    let model = self.lod_mesh.model;
    self.model_uniform_buffer.write_whole_data(&gfx.queue, &[ModelUniform::new(model)]);

    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Voxel meshing render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw voxelized mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(self.lod_mesh.index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &self.lod_mesh.draws {
      render_pass.set_vertex_buffer(0, self.lod_mesh.vertex_buffer.offset::<Vertex>(draw.base_vertex));
      render_pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    render_pass.pop_debug_group();
    drop(render_pass);

    self.debug_renderer.render(gfx, &mut frame, None, self.camera.get_view_projection_matrix() * model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<VoxelMeshing>(Options {
    name: "Voxel meshing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    ..Options::default()
  }).unwrap();
}
