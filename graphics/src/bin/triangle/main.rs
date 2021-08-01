///! Render a single triangle. Mostly made by following: https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/.

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec3;
use wgpu::{Buffer, BufferAddress, CommandBuffer, InputStepMode, RenderPipeline, VertexAttribute, VertexBufferLayout};

use app::{Frame, Gfx, GuiFrame, Os};
use common::input::RawInput;
use gfx::buffer::BufferBuilder;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use graphics::include_shader;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec3,
  col: Vec3,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
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

pub struct Triangle {
  render_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
}

impl app::Application for Triangle {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_layout_label("Triangle pipeline layout")
      .with_label("Triangle render pipeline")
      .build(&gfx.device);
    let vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Triangle static vertex buffer")
      .build_with_data(&gfx.device, VERTICES)
      .buffer;
    Self {
      render_pipeline,
      vertex_buffer,
    }
  }

  type Input = ();

  fn process_input(&mut self, _raw_input: RawInput) -> () {}

  fn render<'a>(&mut self, _os: &Os, _gfx: &Gfx, frame: Frame<'a>, _gui_frame: &GuiFrame, _input: &()) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut render_pass = RenderPassBuilder::new()
      .with_label("Triangle render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw triangle");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..VERTICES.len() as u32, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() { app::run_with_defaults::<Triangle>("Triangle").unwrap(); }