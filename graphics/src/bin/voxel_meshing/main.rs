///! Voxel meshing

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use egui::{color_picker, DragValue, Rgba, Ui};
use egui::color_picker::Alpha;
use ultraviolet::{Mat4, Vec3, Vec4};
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

use crate::density_function::NoiseDensityFunction;
use crate::marching_cubes::MarchingCubes;

mod marching_cubes;
mod density_function;

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

    let noise_density_function = NoiseDensityFunction::new_ridge(32 * 4, -1.0, 1.0);
    let marching_cubes = MarchingCubes::new(noise_density_function, 0.0);
    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(&gfx.device, &marching_cubes.generate());

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


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
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


fn main() {
  app::run::<VoxelMeshing>(Options {
    name: "Voxel meshing".to_string(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}
