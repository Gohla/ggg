use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{UVec3, Vec3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// Chunk trait

pub trait Chunk: Default + Clone + Send + 'static {
  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;
  const CELLS_IN_CHUNK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_CHUNK_USIZE: usize = Self::CELLS_IN_CHUNK as usize;

  const HALF_CELLS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;
  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;

  fn is_empty(&self) -> bool;
  fn vertices(&self) -> &[Vertex];
  fn indices(&self) -> &[u16];

  fn add_vertex(&mut self, vertex: Vertex) -> u16;
  fn add_index(&mut self, index: u16);
  fn clear(&mut self);
}

// Chunk samples

pub enum ChunkSamples<C: Chunk> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:
{
  /// All sampled values in the chunk are exactly `0.0`.
  Zero,
  /// All sampled values in the chunk are positive (i.e., `f32::is_sign_positive() == true`).
  Positive,
  /// All sampled values in the chunk are negative (i.e., `f32::is_sign_negative() == true`).
  Negative,
  /// Sampled values in the chunk are mixed.
  Mixed(ChunkSampleArray<C>),
}

impl<C: Chunk> ChunkSamples<C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:
{
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

pub struct ChunkSampleArray<C: Chunk> where
// This constraint is stating that an array of this size exists. This apparently is necessary because
// VOXELS_IN_CHUNK_USIZE is an unknown constant and the compiler cannot be sure that an array of this size can be made.
// This constraint specifies that the type must exist.
// From: https://stackoverflow.com/questions/66361365/unconstrained-generic-constant-when-adding-const-generics
  [f32; C::VOXELS_IN_CHUNK_USIZE]:
{
  array: [f32; C::VOXELS_IN_CHUNK_USIZE],
}

impl<C: Chunk> ChunkSampleArray<C> where [f32; C::VOXELS_IN_CHUNK_USIZE]: {
  #[inline]
  pub fn new(array: [f32; C::VOXELS_IN_CHUNK_USIZE]) -> Self {
    Self { array }
  }

  #[inline]
  pub fn sample(&self, index: UVec3) -> f32 {
    let index = (index.x + C::VOXELS_IN_CHUNK_ROW * index.y + C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW * index.z) as usize;
    self.array[index]
  }
}

// Chunk

#[derive(Clone, Default, Debug)]
pub struct Chunk16 {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

impl Chunk16 {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_vertices_indices(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
    Self { vertices, indices }
  }
}

impl Chunk for Chunk16 {
  const CELLS_IN_CHUNK_ROW: u32 = 16;

  #[inline]
  fn is_empty(&self) -> bool { self.vertices.is_empty() && self.indices.is_empty() }

  #[inline]
  fn vertices(&self) -> &[Vertex] {
    &self.vertices
  }

  #[inline]
  fn indices(&self) -> &[u16] {
    &self.indices
  }

  #[inline]
  fn add_vertex(&mut self, vertex: Vertex) -> u16 {
    let index = self.vertices.len();
    self.vertices.push(vertex);
    index as u16
  }

  #[inline]
  fn add_index(&mut self, index: u16) {
    self.indices.push(index);
  }

  #[inline]
  fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }
}

// LOD chunk

#[derive(Clone, Default, Debug)]
pub struct LodChunk<C: Chunk> {
  pub regular: C,
  pub transition_lo_x_chunk: C,
  pub transition_hi_x_chunk: C,
  pub transition_lo_y_chunk: C,
  pub transition_hi_y_chunk: C,
  pub transition_lo_z_chunk: C,
  pub transition_hi_z_chunk: C,
}

impl<C: Chunk> LodChunk<C> {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunks(
    regular: C,
    transition_lo_x_chunk: C,
    transition_hi_x_chunk: C,
    transition_lo_y_chunk: C,
    transition_hi_y_chunk: C,
    transition_lo_z_chunk: C,
    transition_hi_z_chunk: C,
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

