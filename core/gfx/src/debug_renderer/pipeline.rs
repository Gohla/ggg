use bytemuck::Pod;
use wgpu::{BindGroupLayout, CommandEncoder, IndexFormat, PolygonMode, PrimitiveTopology, RenderPass, RenderPipeline, ShaderModule, VertexBufferLayout};
use wgpu::util::StagingBelt;

use crate::Gfx;
use crate::growable_buffer::{GrowableBuffer, GrowableBufferBuilder};
use crate::render_pipeline::RenderPipelineBuilder;

pub trait Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static>;
}

pub struct Pipeline<V> {
  render_pipeline: RenderPipeline,
  vertex_buffer: GrowableBuffer<String>,
  index_buffer: GrowableBuffer<String>,
  write_buffers: bool,
  vertices: Vec<V>,
  indices: Vec<u32>,
  label: &'static str,
}

impl<V: Vertex + Pod> Pipeline<V> {
  pub fn new(
    gfx: &Gfx,
    vertex_shader_module: &ShaderModule,
    fragment_shader_module: &ShaderModule,
    uniform_bind_group_layout: &BindGroupLayout,
    multisample_sample_count: u32,
    primitive_topology: PrimitiveTopology,
    polygon_mode: PolygonMode,
    label: &'static str,
  ) -> Self {
    let (_, render_pipeline) = RenderPipelineBuilder::default()
      .with_layout_label(&format!("Debug {} pipeline layout", label))
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_label(&format!("Debug {} render pipeline", label))
      .with_vertex_module(&vertex_shader_module)
      .with_vertex_buffer_layouts(&[V::buffer_layout()])
      .with_primitive_topology(primitive_topology)
      .with_cull_mode(None)
      .with_polygon_mode(polygon_mode)
      .with_fragment_module(&fragment_shader_module)
      .with_surface_premultiplied_alpha_blend_fragment_target(&gfx.surface)
      .with_multisample_count(multisample_sample_count)
      .build(&gfx.device);
    let vertex_buffer = GrowableBufferBuilder::new()
      .with_vertex_usage()
      .with_label(format!("Debug {} vertex buffer", label))
      .create();
    let index_buffer = GrowableBufferBuilder::new()
      .with_index_usage()
      .with_label(format!("Debug {} index buffer", label))
      .create();
    Self {
      render_pipeline,
      vertex_buffer,
      index_buffer,
      write_buffers: false,
      vertices: Vec::default(),
      indices: Vec::default(),
      label,
    }
  }

  #[inline]
  pub fn push_vertex(&mut self, vertex: V) -> u32 {
    let index = self.next_index();
    self.push_index(index);
    self.push_vertex_without_index(vertex);
    self.write_buffers = true;
    index
  }

  #[inline]
  pub fn push_vertices(&mut self, vertices: impl IntoIterator<Item=V>) {
    let base = self.next_index();
    self.vertices.extend(vertices);
    self.indices.extend(base..self.next_index());
    self.write_buffers = true;
  }

  #[inline]
  pub fn push_vertices_and_indices(&mut self, vertices: impl IntoIterator<Item=V>, indices: impl IntoIterator<Item=u32>) {
    let base = self.vertices.len() as u32;
    self.vertices.extend(vertices);
    self.indices.extend(indices.into_iter().map(|idx| base + idx));
    self.write_buffers = true;
  }

  pub fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }

  pub fn write_buffers_if_needed(&mut self, gfx: &Gfx, encoder: &mut CommandEncoder, staging_belt: &mut StagingBelt) {
    if self.write_buffers {
      self.vertex_buffer.write_data(&gfx.device, encoder, staging_belt, &self.vertices);
      self.index_buffer.write_data(&gfx.device, encoder, staging_belt, &self.indices);
    }
    self.write_buffers = false;
  }

  pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
    if let (Some(vertex_buffer), Some(index_buffer)) = (self.vertex_buffer.backing_buffer(), self.index_buffer.backing_buffer()) {
      render_pass.push_debug_group(&format!("Debug draw {}", self.label));
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
      if !self.indices.is_empty() {
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
      } else {
        render_pass.draw(0..self.vertices.len() as u32, 0..1);
      }
      render_pass.pop_debug_group();
    }
  }


  #[inline]
  fn push_vertex_without_index(&mut self, vertex: V) {
    self.vertices.push(vertex);
  }

  #[inline]
  fn next_index(&self) -> u32 {
    self.vertices.len() as u32
  }

  #[inline]
  fn push_index(&mut self, index: u32) {
    self.indices.push(index);
  }
}
