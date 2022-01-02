#![feature(int_log)]

///! Voxel meshing

use bytemuck::{Pod, Zeroable};
use egui::{color_picker, ComboBox, DragValue, Rgba, Ui};
use egui::color_picker::Alpha;
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, PowerPreference, RenderPipeline, ShaderStages};

use app::{Frame, Gfx, GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{CameraInput, CameraSys};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use graphics::include_shader;
use gui_widget::UiWidgetsExt;
use voxel_meshing::marching_cubes::{MarchingCubes, MarchingCubesSettings};
use voxel_meshing::octree::{Octree, OctreeSettings};
use voxel_meshing::vertex::Vertex;
use voxel_meshing::volume::{Noise, NoiseSettings, Sphere, SphereSettings};

pub struct VoxelMeshing {
  camera_sys: CameraSys,
  camera_uniform_buffer: GfxBuffer,
  light_uniform: LightUniform,
  light_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  depth_texture: GfxTexture,
  render_pipeline: RenderPipeline,
  vertex_buffer: GfxBuffer,

  volume_type: VolumeType,
  sphere_settings: SphereSettings,
  noise_settings: NoiseSettings,

  meshing_algorithm_type: MeshingAlgorithmType,
  marching_cubes_settings: MarchingCubesSettings,

  octree_settings: OctreeSettings,
  octree_lod_level: u32,
  vertices: Vec<Vertex>,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

impl app::Application for VoxelMeshing {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;
    let camera_sys = CameraSys::with_defaults_perspective(viewport);

    let depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));

    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .build_with_data(&gfx.device, &[CameraUniform::from_camera_sys(&camera_sys)]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform = LightUniform::new(Vec3::new(0.9, 0.9, 0.9), 0.01, Vec3::new(-0.5, -0.5, -0.5));
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(&gfx.device, &[light_uniform]);
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
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Voxel meshing pipeline layout")
      .with_label("Voxel meshing render pipeline")
      .build(&gfx.device);

    let volume_type = VolumeType::Sphere;
    let sphere_settings = SphereSettings::default();
    let noise_settings = NoiseSettings::default();

    let meshing_algorithm_type = MeshingAlgorithmType::MarchingCubes;
    let marching_cubes_settings = MarchingCubesSettings::default();
    let marching_cubes = MarchingCubes::new(marching_cubes_settings);

    let octree_settings = OctreeSettings::default();
    let octree = Octree::new(octree_settings, Sphere::new(sphere_settings), marching_cubes);
    let mut vertices = Vec::new();
    let octree_lod_level = 0;
    octree.generate_into(octree_lod_level, &mut vertices);
    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(&gfx.device, &vertices);

    Self {
      camera_sys,
      camera_uniform_buffer,
      light_uniform_buffer,
      light_uniform,
      uniform_bind_group,
      depth_texture,
      render_pipeline,
      vertex_buffer,

      volume_type,
      sphere_settings,
      noise_settings,

      meshing_algorithm_type,
      marching_cubes_settings,

      octree_settings,
      octree_lod_level,
      vertices,
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

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta, &gui_frame);
    self.camera_uniform_buffer.write_whole_data(&gfx.queue, &[CameraUniform::from_camera_sys(&self.camera_sys)]);

    egui::Window::new("Voxel Meshing").show(&gui_frame, |ui| {
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
        ui.drag_vec3(0.01, &mut self.light_uniform.direction);
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
            ui.drag_unlabelled(&mut self.noise_settings.lacunarity, 0.01);
            ui.end_row();
            ui.label("Frequency");
            ui.drag_unlabelled(&mut self.noise_settings.frequency, 0.001);
            ui.end_row();
            ui.label("Gain");
            ui.drag_unlabelled(&mut self.noise_settings.gain, 0.01);
            ui.end_row();
            ui.label("Octaves");
            ui.drag_unlabelled(&mut self.noise_settings.octaves, 1);
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
      ui.collapsing_open_with_grid("Octree", "Grid", |ui| {
        ui.label("LOD level");
        ui.drag_unlabelled(&mut self.octree_lod_level, 1); // TODO: limit range
        ui.end_row();
      });
      if ui.button("Generate").clicked() {
        self.generate_vertices();
        self.vertex_buffer = BufferBuilder::new()
          .with_vertex_usage()
          .with_label("Voxel meshing vertex buffer")
          .build_with_data(&gfx.device, &self.vertices);
      }
    });
    self.light_uniform_buffer.write_whole_data(&gfx.queue, &[self.light_uniform]);

    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Voxel meshing render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw voxelized mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..self.vertex_buffer.len as u32, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

impl VoxelMeshing {
  fn generate_vertices(&mut self) {
    self.vertices.clear();
    let meshing_algorithm = MarchingCubes::new(self.marching_cubes_settings);
    match self.volume_type {
      VolumeType::Sphere => Octree::new(self.octree_settings, Sphere::new(self.sphere_settings), meshing_algorithm).generate_into(self.octree_lod_level, &mut self.vertices),
      VolumeType::Noise => Octree::new(self.octree_settings, Noise::new(self.noise_settings), meshing_algorithm).generate_into(self.octree_lod_level, &mut self.vertices),
    };
  }
}


#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum VolumeType {
  Sphere,
  Noise,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum MeshingAlgorithmType {
  MarchingCubes,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
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
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
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


fn main() {
  app::run::<VoxelMeshing>(Options {
    name: "Voxel meshing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
