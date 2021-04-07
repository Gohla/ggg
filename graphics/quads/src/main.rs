use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Isometry3, Mat4, Rotor3, Vec2, Vec3};
use wgpu::{BindGroup, Buffer, BufferAddress, CommandBuffer, include_spirv, IndexFormat, InputStepMode, PipelineLayout, RenderPipeline, ShaderModule, ShaderStage, VertexAttribute, VertexBufferLayout};

use app::{Frame, Gfx, Os, Tick};
use common::input::RawInput;
use common::prelude::ScreenSize;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{CameraInput, CameraSys};
use gfx::prelude::*;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::sampler::SamplerBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use gui::Gui;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec2,
  tex: Vec2,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float2, 1 => Float2];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: InputStepMode::Vertex,
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

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  model: Mat4,
}

impl Instance {
  fn from_isometry(isometry: Isometry3) -> Self { Self { model: isometry.into_homogeneous_matrix() } }

  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![2 => Float4, 3 => Float4, 4 => Float4, 5 => Float4];
    VertexBufferLayout {
      array_stride: size_of::<Instance>() as BufferAddress,
      step_mode: InputStepMode::Instance,
      attributes: ATTRIBUTES,
    }
  }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;

pub struct App {
  camera_sys: CameraSys,

  diffuse_bind_group: BindGroup,

  uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,

  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,

  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,

  depth_texture: GfxTexture,

  vertex_buffer: Buffer,
  index_buffer: Buffer,
  instance_buffer: Buffer,

  gui: Gui,
}

pub struct Input {
  camera: CameraInput,
}

impl app::Application for App {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;
    let camera_sys = CameraSys::with_defaults_orthographic(viewport);

    let (diffuse_bind_group_layout, diffuse_bind_group) = {
      let image = image::load_from_memory(include_bytes!("../../../assets/cobble_stone.bmp")).unwrap().into_rgba8();
      let texture = TextureBuilder::new_from_2d_rgba_image(&image)
        .with_texture_label("Cobblestone diffuse texture")
        .with_texture_view_label("Cobblestone diffuse texture view")
        .build(&gfx.device);
      texture.write_2d_rgba_image(&gfx.queue, image);
      let sampler = SamplerBuilder::new()
        .with_label("Cobblestone diffuse sampler")
        .build(&gfx.device);
      let (view_layout_entry, view_bind_entry) = texture.create_bind_group_entries(0, ShaderStage::FRAGMENT);
      let (sampler_layout_entry, sampler_bind_entry) = sampler.create_bind_group_entries(1, ShaderStage::FRAGMENT);
      CombinedBindGroupLayoutBuilder::new()
        .with_layout_entries(&[view_layout_entry, sampler_layout_entry])
        .with_entries(&[view_bind_entry, sampler_bind_entry])
        .with_layout_label("Cobblestone diffuse bind group layout")
        .with_label("Cobblestone diffuse bind group")
        .build(&gfx.device)
    };

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .build_with_data(&gfx.device, &[Uniform { view_projection: camera_sys.get_view_projection_matrix() }]);
    let (uniform_bind_group_layout, uniform_bind_group) = uniform_buffer.create_uniform_singleton_binding(&gfx.device, ShaderStage::VERTEX);
    let uniform_buffer = uniform_buffer;

    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/quad.vert.spv"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/quad.frag.spv"));

    let depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);

    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&diffuse_bind_group_layout, &uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout(), Instance::buffer_layout()])
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Quads pipeline layout")
      .with_label("Quads render pipeline")
      .build(&gfx.device);
    let vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Static vertex buffer")
      .build_with_data(&gfx.device, VERTICES)
      .buffer;
    let index_buffer = BufferBuilder::new()
      .with_static_index_usage()
      .with_label("Static index buffer")
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
      .with_label("Instance buffer")
      .build_with_data(&gfx.device, &instances)
      .buffer
      ;

    let gui = Gui::new(&gfx.device, gfx.swap_chain.get_texture_format());

    Self {
      camera_sys,

      diffuse_bind_group,

      uniform_buffer,
      uniform_bind_group,

      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,

      _pipeline_layout: pipeline_layout,
      render_pipeline,

      depth_texture,

      vertex_buffer,
      index_buffer,
      instance_buffer,

      gui,
    }
  }

  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    self.gui.process_input(&input);
    Input { camera }
  }

  fn simulate(&mut self, _tick: Tick, _input: &Input) {}

  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, inner_screen_size: ScreenSize) {
    let viewport = inner_screen_size.physical;
    self.camera_sys.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);
  }

  fn render<'a>(&mut self, os: &Os, gfx: &Gfx, frame: Frame<'a>, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform { view_projection: self.camera_sys.get_view_projection_matrix() }]);

    let mut ui = self.gui.begin_frame(os.window.get_inner_size(), frame.time.elapsed.as_s(), frame.time.delta.as_s());
    ui.heading("My egui Application");

    let mut encoder = gfx.device.create_default_command_encoder();
    {
      let mut render_pass = RenderPassBuilder::new()
        .with_depth_texture(&self.depth_texture.view)
        .begin_render_pass_for_swap_chain_with_clear(&mut encoder, &frame.output_texture);
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
      render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
      render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..NUM_INSTANCES);
    }
    self.gui.render(os.window.get_inner_size(), &gfx.device, &gfx.queue, &mut encoder, &frame.output_texture);

    Box::new(std::iter::once(encoder.finish()))
  }
}

fn main() { app::run_with_defaults::<App>("Quads").unwrap(); }