use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{UVec3, Vec3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// Constants

pub const CELLS_IN_CHUNK_ROW: u32 = 16;
pub const CELLS_IN_CHUNK_ROW_F32: f32 = CELLS_IN_CHUNK_ROW as f32;
pub const CELLS_IN_CHUNK_ROW_USIZE: usize = CELLS_IN_CHUNK_ROW as usize;
pub const CELLS_IN_CHUNK: u32 = CELLS_IN_CHUNK_ROW * CELLS_IN_CHUNK_ROW * CELLS_IN_CHUNK_ROW;
pub const CELLS_IN_CHUNK_USIZE: usize = CELLS_IN_CHUNK as usize;

pub const HALF_CELLS_IN_CHUNK_ROW: u32 = CELLS_IN_CHUNK_ROW / 2;

pub const VOXELS_IN_CHUNK_ROW: u32 = CELLS_IN_CHUNK_ROW + 1;
pub const VOXELS_IN_CHUNK_ROW_USIZE: usize = VOXELS_IN_CHUNK_ROW as usize;
pub const VOXELS_IN_CHUNK: u32 = VOXELS_IN_CHUNK_ROW * VOXELS_IN_CHUNK_ROW * VOXELS_IN_CHUNK_ROW;
pub const VOXELS_IN_CHUNK_USIZE: usize = VOXELS_IN_CHUNK as usize;

// Chunk samples

pub enum ChunkSamples {
  /// All sampled values in the chunk are exactly `0.0`.
  Zero,
  /// All sampled values in the chunk are positive (i.e., `f32::is_sign_positive() == true`).
  Positive,
  /// All sampled values in the chunk are negative (i.e., `f32::is_sign_negative() == true`).
  Negative,
  /// Sampled values in the chunk are mixed.
  Mixed(ChunkSampleArray),
}

impl ChunkSamples {
  #[inline]
  pub fn sample(&self, index: UVec3) -> f32 {
    use ChunkSamples::*;
    match self {
      Zero => 0.0,
      Positive => 1.0,
      Negative => -1.0,
      Mixed(array) => array.sample(index)
    }
  }
}

pub struct ChunkSampleArray {
  array: [f32; VOXELS_IN_CHUNK_USIZE],
}

impl ChunkSampleArray {
  #[inline]
  pub fn new(array: [f32; VOXELS_IN_CHUNK_USIZE]) -> Self {
    Self { array }
  }

  #[inline]
  pub fn sample(&self, index: UVec3) -> f32 {
    let index = (index.x + VOXELS_IN_CHUNK_ROW * index.y + VOXELS_IN_CHUNK_ROW * VOXELS_IN_CHUNK_ROW * index.z) as usize;
    self.array[index]
  }
}

// Chunk

#[derive(Clone, Default, Debug)]
pub struct Chunk {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

impl Chunk {
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
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }
}

// LOD chunk

#[derive(Clone, Default, Debug)]
pub struct LodChunk {
  pub regular: Chunk,
  pub transition_lo_x_chunk: Chunk,
  pub transition_hi_x_chunk: Chunk,
  pub transition_lo_y_chunk: Chunk,
  pub transition_hi_y_chunk: Chunk,
  pub transition_lo_z_chunk: Chunk,
  pub transition_hi_z_chunk: Chunk,
}

impl LodChunk {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunks(
    regular: Chunk,
    transition_lo_x_chunk: Chunk,
    transition_hi_x_chunk: Chunk,
    transition_lo_y_chunk: Chunk,
    transition_hi_y_chunk: Chunk,
    transition_lo_z_chunk: Chunk,
    transition_hi_z_chunk: Chunk,
  ) -> Self {
    Self {
      regular,
      transition_lo_x_chunk,
      transition_hi_x_chunk,
      transition_lo_y_chunk,
      transition_hi_y_chunk,
      transition_lo_z_chunk,
      transition_hi_z_chunk,
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.regular.clear();
    self.transition_lo_x_chunk.clear();
    self.transition_hi_x_chunk.clear();
    self.transition_lo_y_chunk.clear();
    self.transition_hi_y_chunk.clear();
    self.transition_lo_z_chunk.clear();
    self.transition_hi_z_chunk.clear();
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

