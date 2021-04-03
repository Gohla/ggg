use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec3;
use wgpu::{Buffer, BufferAddress, CommandBuffer, include_spirv, IndexFormat, InputStepMode, PipelineLayout, RenderPipeline, ShaderModule, VertexAttribute, VertexBufferLayout};

use app::{Frame, Gfx, Os, Tick};
use gfx::buffer::DeviceBufferEx;
use gfx::command::DeviceCommandEncoderEx;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use os::input_sys::RawInput;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec3,
  col: Vec3,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float3, 1 => Float3];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: InputStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex { pos: Vec3::new(0.0, 0.5, 0.0), col: Vec3::new(1.0, 0.0, 0.0) },
  Vertex { pos: Vec3::new(-0.5, -0.5, 0.0), col: Vec3::new(0.0, 1.0, 0.0) },
  Vertex { pos: Vec3::new(0.5, -0.5, 0.0), col: Vec3::new(0.0, 0.0, 1.0) },
];

const INDICES: &[u16] = &[ // Indices are not necessary for a triangle, but here for demo purposes anyway.
  0, 1, 2
];

pub struct Triangle {
  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  index_buffer: Buffer,
}

impl app::App for Triangle {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/triangle.vert.spv"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/triangle.frag.spv"));
    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .build(&gfx.device);
    let vertex_buffer = gfx.device.create_vertex_buffer(VERTICES);
    let index_buffer = gfx.device.create_index_buffer(INDICES);
    Self {
      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,
      _pipeline_layout: pipeline_layout,
      render_pipeline,
      vertex_buffer,
      index_buffer,
    }
  }

  fn process_input(&mut self, _raw_input: RawInput) {}

  fn simulate(&mut self, _tick: Tick) {}

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut encoder = gfx.device.create_default_command_encoder();
    {
      let mut render_pass = RenderPassBuilder::new()
        .begin_render_pass_for_swap_chain(&mut encoder, &frame.output_texture);
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
    Box::new(std::iter::once(encoder.finish()))
  }
}

fn main() { app::run_with_defaults::<Triangle>("Triangle").unwrap(); }
