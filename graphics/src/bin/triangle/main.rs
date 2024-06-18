///! Render a single triangle. Mostly made by following: https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/.

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec3;
use wgpu::{BufferAddress, CommandBuffer, RenderPipeline, VertexAttribute, VertexBufferLayout, VertexStepMode};

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use os::Os;

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
      step_mode: VertexStepMode::Vertex,
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
  vertex_buffer: GfxBuffer,
}

impl app::Application for Triangle {
  type Data = ();
  fn new(_os: &Os, gfx: &Gfx, _viewport: ScreenSize, _config: Self::Data) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));
    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Triangle pipeline layout")
      .vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .label("Triangle render pipeline")
      .vertex_module(&vertex_shader_module)
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);
    let vertex_buffer = BufferBuilder::default()
      .label("Triangle static vertex buffer")
      .static_vertex_usage()
      .build_with_data(&gfx.device, VERTICES);
    Self {
      render_pipeline,
      vertex_buffer,
    }
  }

  type Input = ();
  fn process_input(&mut self, _raw_input: RawInput) -> () {}

  fn render(&mut self, RenderInput { gfx_frame, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut pass = gfx_frame.render_pass_builder().label("Triangle render pass").begin();
    pass.push_debug_group("Draw triangle");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.draw(0..VERTICES.len() as u32, 0..1);
    pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Triangle")
    .without_depth_stencil_texture()
    .run::<Triangle>()
    .unwrap();
}
