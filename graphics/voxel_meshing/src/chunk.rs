use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec3;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// Chunk

#[derive(Clone, Default, Debug)]
pub struct Chunk {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

impl Chunk {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_vertices_indices(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
    Self { vertices, indices }
  }

  pub fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }
}

// Vertex

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
  position: Vec3,
}

impl Vertex {
  pub fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
      0 => Float32x3,
    ];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }

  #[inline]
  pub fn new(position: Vec3) -> Self {
    Self { position }
  }
}

