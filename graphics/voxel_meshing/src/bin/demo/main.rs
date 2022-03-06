#![feature(int_log)]

///! Voxel meshing

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, Features, IndexFormat, PowerPreference, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{CameraInput, CameraSys};
use gfx::debug_renderer::DebugRenderer;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use voxel_meshing::chunk::{Vertex};

use crate::mesh_generation::MeshGeneration;
use crate::settings::Settings;

pub mod settings;
pub mod mesh_generation;

pub struct VoxelMeshing {
  settings: Settings,

  camera_sys: CameraSys,

  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  depth_texture: GfxTexture,
  render_pipeline: RenderPipeline,

  mesh_generation: MeshGeneration,
  debug_renderer: DebugRenderer,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

impl app::Application for VoxelMeshing {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let mut settings = Settings::default();
    settings.light_rotation_y_degree = 270.0;
    settings.render_regular_chunks = true;
    settings.render_transition_lo_x_chunks = false;
    settings.render_transition_hi_x_chunks = false;
    settings.render_transition_lo_y_chunks = false;
    settings.render_transition_hi_y_chunks = false;
    settings.render_transition_lo_z_chunks = false;
    settings.render_transition_hi_z_chunks = true;
    settings.debug_render_octree_nodes = true;
    settings.debug_render_octree_node_color = Vec4::new(0.0, 1.0, 0.0, 0.75);
    settings.debug_render_octree_node_empty_color = Vec4::new(1.0, 0.0, 0.0, 0.1);
    settings.octree_settings.lod_factor = 2.0;
    settings.octree_settings.thread_pool_threads = 1;
    settings.auto_update = true;

    let viewport = os.window.get_inner_size().physical;
    let mut camera_sys = CameraSys::with_defaults_perspective(viewport);
    camera_sys.position = Vec3::new(4096.0 / 2.0, 4096.0 / 2.0, -(4096.0 / 2.0) - 200.0);
    camera_sys.panning_speed = 2000.0;
    camera_sys.far = 10000.0;

    let depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));

    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .build_with_data(&gfx.device, &[CameraUniform::from_camera_sys(&camera_sys)]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(&gfx.device, &[settings.light_uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry])
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

    let volume_mesh_manager = settings.create_volume_mesh_manager();
    let mut debug_renderer = DebugRenderer::new(gfx, camera_sys.get_view_projection_matrix());
    let mesh_generation = MeshGeneration::new(camera_sys.position, &settings, volume_mesh_manager, &mut debug_renderer, &gfx.device);

    Self {
      settings,

      camera_sys,

      camera_uniform_buffer,
      light_uniform_buffer,
      uniform_bind_group,
      depth_texture,
      render_pipeline,

      mesh_generation,
      debug_renderer,
    }
  }


  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera_sys.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera_sys.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta, &gui_frame);

    let mut recreate_volume_mesh_manager = false;
    let mut update_volume_mesh_manager = false;
    egui::Window::new("Voxel Meshing").show(&gui_frame, |ui| {
      self.settings.render_gui(ui, &mut self.mesh_generation, &mut recreate_volume_mesh_manager, &mut update_volume_mesh_manager);
    });
    if recreate_volume_mesh_manager {
      self.mesh_generation.set_volume_mesh_manager(self.settings.create_volume_mesh_manager(), self.camera_sys.position, &self.settings, &mut self.debug_renderer, &gfx.device);
    } else if self.settings.auto_update || update_volume_mesh_manager {
      self.mesh_generation.update(self.camera_sys.position, &self.settings, &mut self.debug_renderer, &gfx.device);
    }

    self.camera_uniform_buffer.write_whole_data(&gfx.queue, &[CameraUniform::from_camera_sys(&self.camera_sys)]);
    self.light_uniform_buffer.write_whole_data(&gfx.queue, &[self.settings.light_uniform]);

    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Voxel meshing render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw voxelized mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(self.mesh_generation.index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &self.mesh_generation.draws {
      render_pass.set_vertex_buffer(0, self.mesh_generation.vertex_buffer.offset::<Vertex>(draw.base_vertex));
      render_pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    render_pass.pop_debug_group();
    drop(render_pass);

    self.debug_renderer.render(gfx, &mut frame, self.camera_sys.get_view_projection_matrix());

    Box::new(std::iter::empty())
  }
}

// Uniform data

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct CameraUniform {
  pub position: Vec4,
  pub view_projection: Mat4,
}

impl CameraUniform {
  pub fn from_camera_sys(camera_sys: &CameraSys) -> Self {
    Self {
      position: camera_sys.position.into_homogeneous_point(),
      view_projection: camera_sys.get_view_projection_matrix(),
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct LightUniform {
  pub color: Vec3,
  pub ambient: f32,
  pub direction: Vec3,
  _dummy: f32, // TODO: replace with crevice crate?
}

impl LightUniform {
  pub fn new(color: Vec3, ambient: f32, direction: Vec3) -> Self {
    Self { color, ambient, direction, _dummy: 0.0 }
  }
}

impl Default for LightUniform {
  fn default() -> Self {
    Self::new(Vec3::new(0.9, 0.9, 0.9), 0.01, Vec3::new(-0.5, -0.5, -0.5))
  }
}

// Main

fn main() {
  app::run::<VoxelMeshing>(Options {
    name: "Voxel meshing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    ..Options::default()
  }).unwrap();
}
