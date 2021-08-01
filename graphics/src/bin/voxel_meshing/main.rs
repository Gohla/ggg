///! Voxel meshing

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use egui::{color_picker, DragValue, Rgba, Ui};
use egui::color_picker::Alpha;
use simdnoise::NoiseBuilder;
use ultraviolet::{Mat4, UVec3, Vec3, Vec4};
use wgpu::{BindGroup, BufferAddress, CommandBuffer, InputStepMode, PowerPreference, RenderPipeline, ShaderStage, VertexAttribute, VertexBufferLayout};

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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec3,
  nor: Vec3,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
      0 => Float32x3,
      1 => Float32x3,
    ];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: InputStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }

  #[inline]
  fn new(pos: Vec3, nor: Vec3) -> Self {
    Self { pos, nor }
  }
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
}

impl LightUniform {
  pub fn new(color: Vec3, ambient: f32, direction: Vec3) -> Self {
    Self { color, ambient, direction }
  }
}

pub struct VoxelMeshing {
  camera_sys: CameraSys,
  camera_uniform_buffer: GfxBuffer,
  light_uniform: LightUniform,
  light_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  depth_texture: GfxTexture,
  render_pipeline: RenderPipeline,
  vertex_buffer: GfxBuffer,
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
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStage::VERTEX_FRAGMENT);
    let light_uniform = LightUniform::new(Vec3::new(0.9, 0.9, 0.9), 0.01, Vec3::new(-0.5, -0.5, -0.5));
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(&gfx.device, &[light_uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStage::FRAGMENT);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry])
      .with_layout_label("Voxel meshing uniform bind group layout")
      .with_label("Voxel meshing uniform bind group")
      .build(&gfx.device);

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Voxel meshing pipeline layout")
      .with_label("Voxel meshing render pipeline")
      .build(&gfx.device);

    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(&gfx.device, &VoxelMeshing::marching_cubes(-1.0, 1.0, 0.0));

    Self {
      camera_sys,
      camera_uniform_buffer,
      light_uniform_buffer,
      light_uniform,
      uniform_bind_group,
      depth_texture,
      render_pipeline,
      vertex_buffer,
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
      ui.grid("Light", |mut ui| {
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
  fn marching_cubes(min: f32, max: f32, surface_level: f32) -> Vec<Vertex> {
    let points_per_axis = 32*4;
    let noise = NoiseBuilder::ridge_3d(points_per_axis, points_per_axis, points_per_axis)
      .with_freq(0.05)
      .with_octaves(5)
      .with_gain(2.0)
      .with_seed(1337)
      .with_lacunarity(0.5)
      .generate_scaled(min, max);
    let points_per_axis = points_per_axis as u32;
    let cubes_per_axis = points_per_axis - 1;
    let mut vertices = Vec::new();
    for x in 0..cubes_per_axis {
      for y in 0..cubes_per_axis {
        for z in 0..cubes_per_axis {
          vertices.extend(VoxelMeshing::cube_vertices(&noise, UVec3::new(x, y, z), points_per_axis, surface_level));
        }
      }
    }
    vertices
  }

  #[inline]
  fn cube_vertices(noise: &Vec<f32>, pos: UVec3, points_per_axis: u32, surface_level: f32) -> Vec<Vertex> {
    let corners = [
      pos + UVec3::new(0, 0, 0),
      pos + UVec3::new(1, 0, 0),
      pos + UVec3::new(1, 0, 1),
      pos + UVec3::new(0, 0, 1),
      pos + UVec3::new(0, 1, 0),
      pos + UVec3::new(1, 1, 0),
      pos + UVec3::new(1, 1, 1),
      pos + UVec3::new(0, 1, 1),
    ];

    let mut configuration = 0;
    for (i, corner) in corners.iter().enumerate() {
      let value = VoxelMeshing::noise_value(noise, *corner, points_per_axis);
      if value < surface_level {
        configuration |= 1 << i;
      }
    }

    let mut vertices = Vec::new();
    let edge_indices: &EdgeIndices = &TRIANGULATION[configuration];
    for i in (0..16).step_by(3) {
      if edge_indices[i] == N { break; }
      let a0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 0] as usize] as usize;
      let a1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 0] as usize] as usize;
      let b0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 1] as usize] as usize;
      let b1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 1] as usize] as usize;
      let c0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 2] as usize] as usize;
      let c1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 2] as usize] as usize;
      let pa = Vec3::from(corners[a0] + corners[a1]) * 0.5;
      let pb = Vec3::from(corners[b0] + corners[b1]) * 0.5;
      let pc = Vec3::from(corners[c0] + corners[c1]) * 0.5;
      let n = (pa - pb).cross(pc - pb).normalized();
      vertices.push(Vertex::new(pa, n));
      vertices.push(Vertex::new(pb, n));
      vertices.push(Vertex::new(pc, n));
    }
    vertices
  }

  #[inline]
  fn noise_value(noise: &Vec<f32>, pos: UVec3, points_per_axis: u32) -> f32 {
    noise[(pos.x + (pos.y * points_per_axis) + (pos.z * points_per_axis * points_per_axis)) as usize]
  }
}

type Edge = u8;
type EdgeIndices = [Edge; 16];

/// Value for no index.
const N: Edge = Edge::MAX;

/// 2D triangulation lookup table that goes from configuration (bitwise concatenation of vertex
/// indices) to array of edge indices.
///
/// The first index is the configuration. Since a cube has 8 corners, there are 2^8 = 256 entries.
///
/// The nested array consist of edge indices used to form triangles. Therefore, these always come
/// in pairs of three. No configuration spans more than 15 edges. The value `N` indicates that there
/// are no further edges for this configuration. Every array always ends with one `N` value and
/// therefore always have size 16.
///
/// From: http://paulbourke.net/geometry/polygonise/ and https://www.youtube.com/watch?v=vTMEdHcKgM4
const TRIANGULATION: [EdgeIndices; 256] = [
  [N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [1, 8, 3, 9, 8, 1, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 10, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 1, 2, 10, N, N, N, N, N, N, N, N, N, N],
  [9, 2, 10, 0, 2, 9, N, N, N, N, N, N, N, N, N, N],
  [2, 8, 3, 2, 10, 8, 10, 9, 8, N, N, N, N, N, N, N],
  [3, 11, 2, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 11, 2, 8, 11, 0, N, N, N, N, N, N, N, N, N, N],
  [1, 9, 0, 2, 3, 11, N, N, N, N, N, N, N, N, N, N],
  [1, 11, 2, 1, 9, 11, 9, 8, 11, N, N, N, N, N, N, N],
  [3, 10, 1, 11, 10, 3, N, N, N, N, N, N, N, N, N, N],
  [0, 10, 1, 0, 8, 10, 8, 11, 10, N, N, N, N, N, N, N],
  [3, 9, 0, 3, 11, 9, 11, 10, 9, N, N, N, N, N, N, N],
  [9, 8, 10, 10, 8, 11, N, N, N, N, N, N, N, N, N, N],
  [4, 7, 8, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [4, 3, 0, 7, 3, 4, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, 8, 4, 7, N, N, N, N, N, N, N, N, N, N],
  [4, 1, 9, 4, 7, 1, 7, 3, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 8, 4, 7, N, N, N, N, N, N, N, N, N, N],
  [3, 4, 7, 3, 0, 4, 1, 2, 10, N, N, N, N, N, N, N],
  [9, 2, 10, 9, 0, 2, 8, 4, 7, N, N, N, N, N, N, N],
  [2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, N, N, N, N],
  [8, 4, 7, 3, 11, 2, N, N, N, N, N, N, N, N, N, N],
  [11, 4, 7, 11, 2, 4, 2, 0, 4, N, N, N, N, N, N, N],
  [9, 0, 1, 8, 4, 7, 2, 3, 11, N, N, N, N, N, N, N],
  [4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, N, N, N, N],
  [3, 10, 1, 3, 11, 10, 7, 8, 4, N, N, N, N, N, N, N],
  [1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, N, N, N, N],
  [4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, N, N, N, N],
  [4, 7, 11, 4, 11, 9, 9, 11, 10, N, N, N, N, N, N, N],
  [9, 5, 4, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [9, 5, 4, 0, 8, 3, N, N, N, N, N, N, N, N, N, N],
  [0, 5, 4, 1, 5, 0, N, N, N, N, N, N, N, N, N, N],
  [8, 5, 4, 8, 3, 5, 3, 1, 5, N, N, N, N, N, N, N],
  [1, 2, 10, 9, 5, 4, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 8, 1, 2, 10, 4, 9, 5, N, N, N, N, N, N, N],
  [5, 2, 10, 5, 4, 2, 4, 0, 2, N, N, N, N, N, N, N],
  [2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, N, N, N, N],
  [9, 5, 4, 2, 3, 11, N, N, N, N, N, N, N, N, N, N],
  [0, 11, 2, 0, 8, 11, 4, 9, 5, N, N, N, N, N, N, N],
  [0, 5, 4, 0, 1, 5, 2, 3, 11, N, N, N, N, N, N, N],
  [2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, N, N, N, N],
  [10, 3, 11, 10, 1, 3, 9, 5, 4, N, N, N, N, N, N, N],
  [4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, N, N, N, N],
  [5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, N, N, N, N],
  [5, 4, 8, 5, 8, 10, 10, 8, 11, N, N, N, N, N, N, N],
  [9, 7, 8, 5, 7, 9, N, N, N, N, N, N, N, N, N, N],
  [9, 3, 0, 9, 5, 3, 5, 7, 3, N, N, N, N, N, N, N],
  [0, 7, 8, 0, 1, 7, 1, 5, 7, N, N, N, N, N, N, N],
  [1, 5, 3, 3, 5, 7, N, N, N, N, N, N, N, N, N, N],
  [9, 7, 8, 9, 5, 7, 10, 1, 2, N, N, N, N, N, N, N],
  [10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, N, N, N, N],
  [8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, N, N, N, N],
  [2, 10, 5, 2, 5, 3, 3, 5, 7, N, N, N, N, N, N, N],
  [7, 9, 5, 7, 8, 9, 3, 11, 2, N, N, N, N, N, N, N],
  [9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, N, N, N, N],
  [2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, N, N, N, N],
  [11, 2, 1, 11, 1, 7, 7, 1, 5, N, N, N, N, N, N, N],
  [9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, N, N, N, N],
  [5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, N],
  [11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, N],
  [11, 10, 5, 7, 11, 5, N, N, N, N, N, N, N, N, N, N],
  [10, 6, 5, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 5, 10, 6, N, N, N, N, N, N, N, N, N, N],
  [9, 0, 1, 5, 10, 6, N, N, N, N, N, N, N, N, N, N],
  [1, 8, 3, 1, 9, 8, 5, 10, 6, N, N, N, N, N, N, N],
  [1, 6, 5, 2, 6, 1, N, N, N, N, N, N, N, N, N, N],
  [1, 6, 5, 1, 2, 6, 3, 0, 8, N, N, N, N, N, N, N],
  [9, 6, 5, 9, 0, 6, 0, 2, 6, N, N, N, N, N, N, N],
  [5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, N, N, N, N],
  [2, 3, 11, 10, 6, 5, N, N, N, N, N, N, N, N, N, N],
  [11, 0, 8, 11, 2, 0, 10, 6, 5, N, N, N, N, N, N, N],
  [0, 1, 9, 2, 3, 11, 5, 10, 6, N, N, N, N, N, N, N],
  [5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, N, N, N, N],
  [6, 3, 11, 6, 5, 3, 5, 1, 3, N, N, N, N, N, N, N],
  [0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, N, N, N, N],
  [3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, N, N, N, N],
  [6, 5, 9, 6, 9, 11, 11, 9, 8, N, N, N, N, N, N, N],
  [5, 10, 6, 4, 7, 8, N, N, N, N, N, N, N, N, N, N],
  [4, 3, 0, 4, 7, 3, 6, 5, 10, N, N, N, N, N, N, N],
  [1, 9, 0, 5, 10, 6, 8, 4, 7, N, N, N, N, N, N, N],
  [10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, N, N, N, N],
  [6, 1, 2, 6, 5, 1, 4, 7, 8, N, N, N, N, N, N, N],
  [1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, N, N, N, N],
  [8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, N, N, N, N],
  [7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9, N],
  [3, 11, 2, 7, 8, 4, 10, 6, 5, N, N, N, N, N, N, N],
  [5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, N, N, N, N],
  [0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, N, N, N, N],
  [9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6, N],
  [8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, N, N, N, N],
  [5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11, N],
  [0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, N],
  [6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, N, N, N, N],
  [10, 4, 9, 6, 4, 10, N, N, N, N, N, N, N, N, N, N],
  [4, 10, 6, 4, 9, 10, 0, 8, 3, N, N, N, N, N, N, N],
  [10, 0, 1, 10, 6, 0, 6, 4, 0, N, N, N, N, N, N, N],
  [8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, N, N, N, N],
  [1, 4, 9, 1, 2, 4, 2, 6, 4, N, N, N, N, N, N, N],
  [3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, N, N, N, N],
  [0, 2, 4, 4, 2, 6, N, N, N, N, N, N, N, N, N, N],
  [8, 3, 2, 8, 2, 4, 4, 2, 6, N, N, N, N, N, N, N],
  [10, 4, 9, 10, 6, 4, 11, 2, 3, N, N, N, N, N, N, N],
  [0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, N, N, N, N],
  [3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, N, N, N, N],
  [6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, N],
  [9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, N, N, N, N],
  [8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, N],
  [3, 11, 6, 3, 6, 0, 0, 6, 4, N, N, N, N, N, N, N],
  [6, 4, 8, 11, 6, 8, N, N, N, N, N, N, N, N, N, N],
  [7, 10, 6, 7, 8, 10, 8, 9, 10, N, N, N, N, N, N, N],
  [0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, N, N, N, N],
  [10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, N, N, N, N],
  [10, 6, 7, 10, 7, 1, 1, 7, 3, N, N, N, N, N, N, N],
  [1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, N, N, N, N],
  [2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9, N],
  [7, 8, 0, 7, 0, 6, 6, 0, 2, N, N, N, N, N, N, N],
  [7, 3, 2, 6, 7, 2, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, N, N, N, N],
  [2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, N],
  [1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, N],
  [11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, N, N, N, N],
  [8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, N],
  [0, 9, 1, 11, 6, 7, N, N, N, N, N, N, N, N, N, N],
  [7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, N, N, N, N],
  [7, 11, 6, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [7, 6, 11, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 8, 11, 7, 6, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, 11, 7, 6, N, N, N, N, N, N, N, N, N, N],
  [8, 1, 9, 8, 3, 1, 11, 7, 6, N, N, N, N, N, N, N],
  [10, 1, 2, 6, 11, 7, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 10, 3, 0, 8, 6, 11, 7, N, N, N, N, N, N, N],
  [2, 9, 0, 2, 10, 9, 6, 11, 7, N, N, N, N, N, N, N],
  [6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, N, N, N, N],
  [7, 2, 3, 6, 2, 7, N, N, N, N, N, N, N, N, N, N],
  [7, 0, 8, 7, 6, 0, 6, 2, 0, N, N, N, N, N, N, N],
  [2, 7, 6, 2, 3, 7, 0, 1, 9, N, N, N, N, N, N, N],
  [1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, N, N, N, N],
  [10, 7, 6, 10, 1, 7, 1, 3, 7, N, N, N, N, N, N, N],
  [10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, N, N, N, N],
  [0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, N, N, N, N],
  [7, 6, 10, 7, 10, 8, 8, 10, 9, N, N, N, N, N, N, N],
  [6, 8, 4, 11, 8, 6, N, N, N, N, N, N, N, N, N, N],
  [3, 6, 11, 3, 0, 6, 0, 4, 6, N, N, N, N, N, N, N],
  [8, 6, 11, 8, 4, 6, 9, 0, 1, N, N, N, N, N, N, N],
  [9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, N, N, N, N],
  [6, 8, 4, 6, 11, 8, 2, 10, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, N, N, N, N],
  [4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, N, N, N, N],
  [10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, N],
  [8, 2, 3, 8, 4, 2, 4, 6, 2, N, N, N, N, N, N, N],
  [0, 4, 2, 4, 6, 2, N, N, N, N, N, N, N, N, N, N],
  [1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, N, N, N, N],
  [1, 9, 4, 1, 4, 2, 2, 4, 6, N, N, N, N, N, N, N],
  [8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, N, N, N, N],
  [10, 1, 0, 10, 0, 6, 6, 0, 4, N, N, N, N, N, N, N],
  [4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, N],
  [10, 9, 4, 6, 10, 4, N, N, N, N, N, N, N, N, N, N],
  [4, 9, 5, 7, 6, 11, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 4, 9, 5, 11, 7, 6, N, N, N, N, N, N, N],
  [5, 0, 1, 5, 4, 0, 7, 6, 11, N, N, N, N, N, N, N],
  [11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, N, N, N, N],
  [9, 5, 4, 10, 1, 2, 7, 6, 11, N, N, N, N, N, N, N],
  [6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, N, N, N, N],
  [7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, N, N, N, N],
  [3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, N],
  [7, 2, 3, 7, 6, 2, 5, 4, 9, N, N, N, N, N, N, N],
  [9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, N, N, N, N],
  [3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, N, N, N, N],
  [6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, N],
  [9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, N, N, N, N],
  [1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, N],
  [4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, N],
  [7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, N, N, N, N],
  [6, 9, 5, 6, 11, 9, 11, 8, 9, N, N, N, N, N, N, N],
  [3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, N, N, N, N],
  [0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, N, N, N, N],
  [6, 11, 3, 6, 3, 5, 5, 3, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, N, N, N, N],
  [0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, N],
  [11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5, N],
  [6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, N, N, N, N],
  [5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, N, N, N, N],
  [9, 5, 6, 9, 6, 0, 0, 6, 2, N, N, N, N, N, N, N],
  [1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8, N],
  [1, 5, 6, 2, 1, 6, N, N, N, N, N, N, N, N, N, N],
  [1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6, N],
  [10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, N, N, N, N],
  [0, 3, 8, 5, 6, 10, N, N, N, N, N, N, N, N, N, N],
  [10, 5, 6, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [11, 5, 10, 7, 5, 11, N, N, N, N, N, N, N, N, N, N],
  [11, 5, 10, 11, 7, 5, 8, 3, 0, N, N, N, N, N, N, N],
  [5, 11, 7, 5, 10, 11, 1, 9, 0, N, N, N, N, N, N, N],
  [10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, N, N, N, N],
  [11, 1, 2, 11, 7, 1, 7, 5, 1, N, N, N, N, N, N, N],
  [0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, N, N, N, N],
  [9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, N, N, N, N],
  [7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, N],
  [2, 5, 10, 2, 3, 5, 3, 7, 5, N, N, N, N, N, N, N],
  [8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, N, N, N, N],
  [9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, N, N, N, N],
  [9, 8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2, N],
  [1, 3, 5, 3, 7, 5, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 7, 0, 7, 1, 1, 7, 5, N, N, N, N, N, N, N],
  [9, 0, 3, 9, 3, 5, 5, 3, 7, N, N, N, N, N, N, N],
  [9, 8, 7, 5, 9, 7, N, N, N, N, N, N, N, N, N, N],
  [5, 8, 4, 5, 10, 8, 10, 11, 8, N, N, N, N, N, N, N],
  [5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, N, N, N, N],
  [0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, N, N, N, N],
  [10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, N],
  [2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, N, N, N, N],
  [0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, N],
  [0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, N],
  [9, 4, 5, 2, 11, 3, N, N, N, N, N, N, N, N, N, N],
  [2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, N, N, N, N],
  [5, 10, 2, 5, 2, 4, 4, 2, 0, N, N, N, N, N, N, N],
  [3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, N],
  [5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, N, N, N, N],
  [8, 4, 5, 8, 5, 3, 3, 5, 1, N, N, N, N, N, N, N],
  [0, 4, 5, 1, 0, 5, N, N, N, N, N, N, N, N, N, N],
  [8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, N, N, N, N],
  [9, 4, 5, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [4, 11, 7, 4, 9, 11, 9, 10, 11, N, N, N, N, N, N, N],
  [0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, N, N, N, N],
  [1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, N, N, N, N],
  [3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, N],
  [4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, N, N, N, N],
  [9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, N],
  [11, 7, 4, 11, 4, 2, 2, 4, 0, N, N, N, N, N, N, N],
  [11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, N, N, N, N],
  [2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, N, N, N, N],
  [9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, N],
  [3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10, N],
  [1, 10, 2, 8, 7, 4, N, N, N, N, N, N, N, N, N, N],
  [4, 9, 1, 4, 1, 7, 7, 1, 3, N, N, N, N, N, N, N],
  [4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, N, N, N, N],
  [4, 0, 3, 7, 4, 3, N, N, N, N, N, N, N, N, N, N],
  [4, 8, 7, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [9, 10, 8, 10, 11, 8, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 9, 3, 9, 11, 11, 9, 10, N, N, N, N, N, N, N],
  [0, 1, 10, 0, 10, 8, 8, 10, 11, N, N, N, N, N, N, N],
  [3, 1, 10, 11, 3, 10, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 11, 1, 11, 9, 9, 11, 8, N, N, N, N, N, N, N],
  [3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, N, N, N, N],
  [0, 2, 11, 8, 0, 11, N, N, N, N, N, N, N, N, N, N],
  [3, 2, 11, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 8, 2, 8, 10, 10, 8, 9, N, N, N, N, N, N, N],
  [9, 10, 2, 0, 9, 2, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, N, N, N, N],
  [1, 10, 2, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [1, 3, 8, 9, 1, 8, N, N, N, N, N, N, N, N, N, N],
  [0, 9, 1, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 3, 8, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N]
];

const CORNER_INDEX_A_FROM_EDGE: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3];
const CORNER_INDEX_B_FROM_EDGE: [u8; 12] = [1, 2, 3, 0, 5, 6, 7, 4, 4, 5, 6, 7];


fn main() {
  app::run::<VoxelMeshing>(Options {
    name: "Voxel meshing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
