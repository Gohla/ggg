use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec2};
use wgpu::{BindGroup, Buffer, BufferAddress, CommandBuffer, include_spirv, IndexFormat, InputStepMode, PipelineLayout, RenderPipeline, ShaderModule, ShaderStage, VertexAttribute, VertexBufferLayout};

use app::{Frame, Gfx, Os, Tick};
use common::input::RawInput;
use common::prelude::ScreenSize;
use gfx::buffer::{BufferEx, DeviceBufferEx};
use gfx::camera::{CameraInput, CameraSys};
use gfx::command::DeviceCommandEncoderEx;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::Texture2dRgbaBuilder;

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

pub struct App {
  camera_sys: CameraSys,

  diffuse_bind_group: BindGroup,

  uniform_buffer: Buffer,
  uniform_bind_group: BindGroup,

  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,

  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,

  vertex_buffer: Buffer,
  index_buffer: Buffer,

}

pub struct Input {
  camera: CameraInput,
}

impl app::Application for App {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let camera_sys = CameraSys::new(os.window.get_inner_size().physical);

    let diffuse_image = image::load_from_memory(include_bytes!("../../../assets/cobble_stone.bmp")).unwrap();
    let (_diffuse, diffuse_bind_group_layout, diffuse_bind_group) = Texture2dRgbaBuilder::new(diffuse_image.into_rgba8())
      .build_with_default_bind_group(&gfx.device, &gfx.queue);

    let uniform_buffer = gfx.device.create_uniform_buffer(&[Uniform { view_projection: camera_sys.view_projection_matrix() }]);
    let (uniform_bind_group_layout, uniform_bind_group) = uniform_buffer.create_binding(&gfx.device, ShaderStage::VERTEX);
    let uniform_buffer = uniform_buffer.into_inner();

    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/quad.vert.spv"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/quad.frag.spv"));

    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&diffuse_bind_group_layout, &uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .build(&gfx.device);
    let vertex_buffer = gfx.device.create_static_vertex_buffer(VERTICES);
    let index_buffer = gfx.device.create_static_index_buffer(INDICES);

    Self {
      camera_sys,

      diffuse_bind_group,

      uniform_buffer,
      uniform_bind_group,

      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,

      _pipeline_layout: pipeline_layout,
      render_pipeline,

      vertex_buffer,
      index_buffer,

    }
  }

  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn simulate(&mut self, _tick: Tick, _input: &Input) {}

  fn screen_resize(&mut self, _os: &Os, _gfx: &Gfx, inner_screen_size: ScreenSize) {
    self.camera_sys.set_viewport(inner_screen_size.physical);
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta);
    self.uniform_buffer.write(&gfx.queue, &[Uniform { view_projection: self.camera_sys.view_projection_matrix() }]);

    let mut encoder = gfx.device.create_default_command_encoder();
    {
      let mut render_pass = RenderPassBuilder::new()
        .begin_render_pass_for_swap_chain(&mut encoder, &frame.output_texture);
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
      render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
      render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    Box::new(std::iter::once(encoder.finish()))
  }
}

fn main() { app::run_with_defaults::<App>("Quads").unwrap(); }
