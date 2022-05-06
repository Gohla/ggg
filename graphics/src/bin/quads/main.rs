///! Render a couple of quads. Mostly made by following: https://sotrh.github.io/learn-wgpu/beginner/tutorial8-depth/.

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use ultraviolet::{Isometry3, Mat4, Rotor3, Vec2, Vec3};
use wgpu::{BindGroup, Buffer, BufferAddress, CommandBuffer, IndexFormat, RenderPipeline, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};

use app::{GuiFrame, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraDebugging, CameraInput, CameraSettings};
use gfx::prelude::*;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::TextureBuilder;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec2,
  tex: Vec2,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex { pos: Vec2::new(-0.5, -0.5), tex: Vec2::new(0.0, 1.0) },
  Vertex { pos: Vec2::new(0.5, -0.5), tex: Vec2::new(1.0, 1.0) },
  Vertex { pos: Vec2::new(-0.5, 0.5), tex: Vec2::new(0.0, 0.0) },
  Vertex { pos: Vec2::new(0.5, 0.5), tex: Vec2::new(1.0, 0.0) },
];

const INDICES: &[u16] = &[
  0, 1, 2,
  1, 3, 2
];

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}

impl Uniform {
  pub fn from_camera(camera: &Camera) -> Self {
    Self {
      view_projection: camera.get_view_projection_matrix(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  model: Mat4,
}

impl Instance {
  fn from_isometry(isometry: Isometry3) -> Self { Self { model: isometry.into_homogeneous_matrix() } }

  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];
    VertexBufferLayout {
      array_stride: size_of::<Instance>() as BufferAddress,
      step_mode: VertexStepMode::Instance,
      attributes: ATTRIBUTES,
    }
  }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;

pub struct Quads {
  camera_settings: CameraSettings,
  camera_debugging: CameraDebugging,
  camera: Camera,

  diffuse_bind_group: BindGroup,

  uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,

  render_pipeline: RenderPipeline,

  vertex_buffer: Buffer,
  index_buffer: Buffer,
  instance_buffer: Buffer,
}

pub struct Input {
  camera: CameraInput,
}

impl app::Application for Quads {
  type Config = ();

  fn new(os: &Os, gfx: &Gfx, _config: Self::Config) -> Self {
    let camera_settings = CameraSettings::with_defaults_arcball_perspective();
    let camera_debugging = CameraDebugging::with_default_settings(camera_settings);
    let camera = Camera::new(os.window.get_inner_size().physical);

    let (diffuse_bind_group_layout, diffuse_bind_group) = {
      let image = image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/cobble_stone_1.png")).unwrap().into_rgba8();
      let texture = TextureBuilder::new_from_2d_rgba_image(&image)
        .with_texture_label("Cobblestone diffuse texture")
        .with_texture_view_label("Cobblestone diffuse texture view")
        .build(&gfx.device);
      texture.write_2d_rgba_image(&gfx.queue, image);
      let sampler = SamplerBuilder::new()
        .with_label("Cobblestone diffuse sampler")
        .build(&gfx.device);
      let (view_layout_entry, view_bind_entry) = texture.create_default_float_2d_bind_group_entries(0, ShaderStages::FRAGMENT);
      let (sampler_layout_entry, sampler_bind_entry) = sampler.create_bind_group_entries(1, ShaderStages::FRAGMENT);
      CombinedBindGroupLayoutBuilder::new()
        .with_layout_entries(&[view_layout_entry, sampler_layout_entry])
        .with_entries(&[view_bind_entry, sampler_bind_entry])
        .with_layout_label("Cobblestone diffuse bind group layout")
        .with_label("Cobblestone diffuse bind group")
        .build(&gfx.device)
    };

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .build_with_data(&gfx.device, &[Uniform { view_projection: camera.get_view_projection_matrix() }]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Quads uniform bind group layout")
      .with_label("Quads uniform bind group")
      .build(&gfx.device);
    let uniform_buffer = uniform_buffer;

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("frag"));

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&diffuse_bind_group_layout, &uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout(), Instance::buffer_layout()])
      .with_depth_texture(gfx.depth_stencil_format().unwrap())
      .with_layout_label("Quads pipeline layout")
      .with_label("Quads render pipeline")
      .build(&gfx.device);
    let vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Quad static vertex buffer")
      .build_with_data(&gfx.device, VERTICES)
      .buffer;
    let index_buffer = BufferBuilder::new()
      .with_static_index_usage()
      .with_label("Quad static index buffer")
      .build_with_data(&gfx.device, INDICES)
      .buffer;
    let instances: Vec<Instance> = (0..NUM_INSTANCES_PER_ROW).flat_map(|y| {
      let y = y as f32 - (NUM_INSTANCES_PER_ROW as f32 / 2.0);
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let x = x as f32 - (NUM_INSTANCES_PER_ROW as f32 / 2.0);
        let translation = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
        let rotation = Rotor3::from_euler_angles(0.0, x, y);
        Instance::from_isometry(Isometry3::new(translation, rotation))
      })
    }).collect();
    let instance_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Quads instance buffer")
      .build_with_data(&gfx.device, &instances)
      .buffer
      ;

    Self {
      camera_settings,
      camera_debugging,
      camera,

      diffuse_bind_group,

      uniform_buffer,
      uniform_bind_group,

      render_pipeline,

      vertex_buffer,
      index_buffer,
      instance_buffer,
    }
  }


  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, screen_size: ScreenSize) {
    self.camera.set_viewport(screen_size.physical);
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
    self.camera_debugging.show_debugging_gui_window(&gui_frame, &self.camera, &mut self.camera_settings);
    self.camera.update(&mut self.camera_settings, &input.camera, frame.time.delta);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform::from_camera(&self.camera)]);

    egui::Window::new("Quads").show(&gui_frame, |ui| {
      ui.label("Hello, world!");
    });

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Quads render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, &mut frame, true);
    render_pass.push_debug_group("Draw quads");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
    render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..NUM_INSTANCES);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() { app::run_with_defaults::<Quads>("Quads").unwrap(); }
