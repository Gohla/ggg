#![feature(int_log)]

///! Voxel meshing

use bytemuck::{Pod, Zeroable};
use egui::{color_picker, ComboBox, DragValue, Rgba, Ui};
use egui::color_picker::Alpha;
use ultraviolet::{Mat4, Rotor3, Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, Device, Features, PowerPreference, RenderPipeline, ShaderStages};

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
use gui_widget::UiWidgetsExt;
use voxel_meshing::marching_cubes::{MarchingCubes, MarchingCubesSettings};
use voxel_meshing::octree::{Octree, OctreeSettings, VolumeMeshManager};
use voxel_meshing::vertex::Vertex;
use voxel_meshing::volume::{Noise, NoiseSettings, Sphere, SphereSettings};

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
      .with_default_alpha_blending_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Voxel meshing pipeline layout")
      .with_label("Voxel meshing render pipeline")
      .build(&gfx.device);

    let volume_mesh_manager = settings.create_volume_mesh_manager();
    let mut debug_renderer = DebugRenderer::new(gfx, camera_sys.get_view_projection_matrix());
    let mesh_generation = MeshGeneration::new(volume_mesh_manager, &mut debug_renderer, camera_sys.position, &gfx.device);

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

    egui::Window::new("Voxel Meshing").show(&gui_frame, |ui| {
      self.settings.render_gui(ui, &mut self.mesh_generation);
      if ui.button("Update").clicked() {
        self.mesh_generation.update(self.camera_sys.position, &mut self.debug_renderer, &gfx.device);
      }
    });

    self.camera_uniform_buffer.write_whole_data(&gfx.queue, &[CameraUniform::from_camera_sys(&self.camera_sys)]);
    self.light_uniform_buffer.write_whole_data(&gfx.queue, &[self.settings.light_uniform]);

    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Voxel meshing render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw voxelized mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.mesh_generation.vertex_buffer.slice(..));
    render_pass.draw(0..self.mesh_generation.vertex_buffer.len as u32, 0..1);
    render_pass.pop_debug_group();
    drop(render_pass);

    self.debug_renderer.render(gfx, &mut frame, self.camera_sys.get_view_projection_matrix());

    Box::new(std::iter::empty())
  }
}

// Volume-mesh management

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
enum VolumeType {
  Sphere,
  Noise,
}

impl Default for VolumeType {
  fn default() -> Self { Self::Sphere }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
enum MeshingAlgorithmType {
  MarchingCubes,
}

impl Default for MeshingAlgorithmType {
  fn default() -> Self { Self::MarchingCubes }
}

#[derive(Copy, Clone, Default, Debug)]
struct Settings {
  light_rotation_x_degree: f32,
  light_rotation_y_degree: f32,
  light_rotation_z_degree: f32,
  light_uniform: LightUniform,

  volume_type: VolumeType,
  sphere_settings: SphereSettings,
  noise_settings: NoiseSettings,

  meshing_algorithm_type: MeshingAlgorithmType,
  marching_cubes_settings: MarchingCubesSettings,

  octree_settings: OctreeSettings,
}

impl Settings {
  fn create_volume_mesh_manager(&self) -> Box<dyn VolumeMeshManager> {
    let meshing_algorithm = MarchingCubes::new(self.marching_cubes_settings);
    match self.volume_type {
      VolumeType::Sphere => Box::new(Octree::new(self.octree_settings, Sphere::new(self.sphere_settings), meshing_algorithm)),
      VolumeType::Noise => Box::new(Octree::new(self.octree_settings, Noise::new(self.noise_settings), meshing_algorithm)),
    }
  }

  fn render_gui(&mut self, ui: &mut Ui, mesh_generation: &mut MeshGeneration) {
    ui.collapsing_open_with_grid("Directional Light", "Grid", |mut ui| {
      ui.label("Color");
      let mut color = Rgba::from_rgba_premultiplied(self.light_uniform.color.x, self.light_uniform.color.y, self.light_uniform.color.z, 0.0).into();
      color_picker::color_edit_button_srgba(&mut ui, &mut color, Alpha::Opaque);
      let color: Rgba = color.into();
      self.light_uniform.color = Vec3::new(color.r(), color.g(), color.b());
      ui.end_row();
      ui.label("Ambient");
      ui.add(DragValue::new(&mut self.light_uniform.ambient).speed(0.001).clamp_range(0.0..=1.0));
      ui.end_row();
      ui.label("Direction");
      ui.drag("x: ", &mut self.light_rotation_x_degree, 0.5);
      ui.drag("y: ", &mut self.light_rotation_y_degree, 0.5);
      ui.drag("z: ", &mut self.light_rotation_z_degree, 0.5);
      self.light_uniform.direction = Rotor3::from_euler_angles((self.light_rotation_z_degree % 360.0).to_radians(), (self.light_rotation_x_degree % 360.0).to_radians(), (self.light_rotation_y_degree % 360.0).to_radians()) * Vec3::one();
      ui.end_row();
    });
    ui.collapsing_open_with_grid("Volume", "Grid", |ui| {
      ui.label("Type");
      ComboBox::from_id_source("Type")
        .selected_text(format!("{:?}", self.volume_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.volume_type, VolumeType::Sphere, "Sphere");
          ui.selectable_value(&mut self.volume_type, VolumeType::Noise, "Noise");
        });
      ui.end_row();
      match self.volume_type {
        VolumeType::Sphere => {
          ui.label("Radius");
          ui.drag_unlabelled(&mut self.sphere_settings.radius, 0.1);
          ui.end_row();
        }
        VolumeType::Noise => {
          ui.label("Seed");
          ui.drag_unlabelled(&mut self.noise_settings.seed, 1);
          ui.end_row();
          ui.label("Lacunarity");
          ui.drag_unlabelled_range(&mut self.noise_settings.lacunarity, 0.01, 0.0..=10.0);
          ui.end_row();
          ui.label("Frequency");
          ui.drag_unlabelled_range(&mut self.noise_settings.frequency, 0.001, 0.0..=1.0);
          ui.end_row();
          ui.label("Gain");
          ui.drag_unlabelled_range(&mut self.noise_settings.gain, 0.01, 0.0..=10.0);
          ui.end_row();
          ui.label("Octaves");
          ui.drag_unlabelled_range(&mut self.noise_settings.octaves, 1, 0..=10);
          ui.end_row();
          ui.label("Minimum density");
          ui.drag_unlabelled(&mut self.noise_settings.min, 0.01);
          ui.end_row();
          ui.label("Maximum density");
          ui.drag_unlabelled(&mut self.noise_settings.max, 0.01);
          ui.end_row();
        }
      }
    });
    ui.collapsing_open_with_grid("Meshing Algorithm", "Grid", |ui| {
      ui.label("Type");
      ComboBox::from_id_source("Type")
        .selected_text(format!("{:?}", self.meshing_algorithm_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.meshing_algorithm_type, MeshingAlgorithmType::MarchingCubes, "Marching Cubes");
        });
      ui.end_row();
      match self.meshing_algorithm_type {
        MeshingAlgorithmType::MarchingCubes => {
          ui.label("Surface level");
          ui.drag_unlabelled(&mut self.marching_cubes_settings.surface_level, 0.01);
          ui.end_row();
        }
      }
    });
    ui.collapsing_open_with_grid("Volume mesh manager", "Grid", |ui| {
      ui.label("LOD factor");
      ui.drag_unlabelled_range(mesh_generation.volume_mesh_manager.get_lod_factor_mut(), 0.1, 0.0..=4.0);
      ui.end_row();
      ui.label("Max LOD level");
      ui.monospace(format!("{}", mesh_generation.volume_mesh_manager.get_max_lod_level()));
      ui.end_row();
      ui.label("# vertices");
      ui.monospace(format!("{}", mesh_generation.vertices.len()));
      ui.end_row();
      ui.label("Vertex buffer size");
      ui.monospace(format!("{}", mesh_generation.vertex_buffer.size));
      ui.end_row();
    });
  }
}

// Mesh generation

struct MeshGeneration {
  volume_mesh_manager: Box<dyn VolumeMeshManager>,
  vertices: Vec<Vertex>,
  vertex_buffer: GfxBuffer,
}

impl MeshGeneration {
  fn new(mut volume_mesh_manager: Box<dyn VolumeMeshManager>, debug_renderer: &mut DebugRenderer, position: Vec3, device: &Device) -> Self {
    let mut vertices = Vec::new();
    let vertex_buffer = Self::do_update(&mut vertices, debug_renderer, &mut *volume_mesh_manager, position, device);
    Self { volume_mesh_manager, vertices, vertex_buffer }
  }

  fn update(&mut self, position: Vec3, debug_renderer: &mut DebugRenderer, device: &Device) {
    self.vertex_buffer = Self::do_update(&mut self.vertices, debug_renderer, &mut *self.volume_mesh_manager, position, device);
  }

  fn do_update(vertices: &mut Vec<Vertex>, debug_renderer: &mut DebugRenderer, volume_mesh_manager: &mut dyn VolumeMeshManager, position: Vec3, device: &Device) -> GfxBuffer {
    vertices.clear();
    debug_renderer.clear();
    let color = Vec4::new(0.0, 1.0, 0.0, 0.5);
    for (aabb, (chunk_vertices, filled)) in volume_mesh_manager.update(position) {
      if *filled {
        vertices.extend(chunk_vertices);
        debug_renderer.draw_cube(aabb.min().into(), aabb.size() as f32, color);
      }
    }
    BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(device, &vertices)
  }
}

// Uniform data

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
struct CameraUniform {
  position: Vec4,
  view_projection: Mat4,
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
struct LightUniform {
  color: Vec3,
  ambient: f32,
  direction: Vec3,
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
    graphics_device_features: Features::POLYGON_MODE_LINE,
    ..Options::default()
  }).unwrap();
}
