use std::mem::size_of;

use ultraviolet::Vec3;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

use bytemuck::{Pod, Zeroable};

// Mesh

#[derive(Clone, Default, Debug)]
pub struct ChunkMesh {
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
}

impl ChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_vertices_indices(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
    Self { vertices, indices }
  }


  #[inline]
  pub fn is_empty(&self) -> bool { self.vertices.is_empty() && self.indices.is_empty() }

  #[inline]
  pub fn vertices(&self) -> &[Vertex] {
    &self.vertices
  }

  #[inline]
  pub fn indices(&self) -> &[u16] {
    &self.indices
  }


  #[inline]
  pub fn push_vertex(&mut self, vertex: Vertex) -> u16 {
    let index = self.vertices.len();
    self.vertices.push(vertex);
    index as u16
  }

  #[inline]
  pub fn push_index(&mut self, index: u16) {
    self.indices.push(index);
  }

  #[inline]
  pub fn extend_indices_from_slice(&mut self, indices: &[u16]) {
    self.indices.extend_from_slice(indices);
  }

  #[inline]
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }
}


// Vertex

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
  pub position: Vec3,
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
