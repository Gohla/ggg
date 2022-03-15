use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{UVec3, Vec3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// Chunk size

pub trait ChunkSize: Default + Copy + Clone + Send + 'static {
  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;
  const CELLS_IN_DECK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_DECK_USIZE: usize = Self::CELLS_IN_DECK as usize;
  const CELLS_IN_CHUNK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_CHUNK_USIZE: usize = Self::CELLS_IN_CHUNK as usize;

  const HALF_CELLS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;
  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct GenericChunkSize<const SIZE: u32>;

impl<const SIZE: u32> ChunkSize for GenericChunkSize<SIZE> {
  const CELLS_IN_CHUNK_ROW: u32 = SIZE;
}

pub type ChunkSize16 = GenericChunkSize<16>;

// Chunk samples

pub enum ChunkSamples<C: ChunkSize> where
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

impl<C: ChunkSize> ChunkSamples<C> where
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

#[derive(Copy, Clone, Debug)]
pub struct ChunkSampleArray<C: ChunkSize> where
// This constraint is stating that an array of this size exists. This apparently is necessary because
// VOXELS_IN_CHUNK_USIZE is an unknown constant and the compiler cannot be sure that an array of this size can be made.
// This constraint specifies that the type must exist.
// From: https://stackoverflow.com/questions/66361365/unconstrained-generic-constant-when-adding-const-generics
  [f32; C::VOXELS_IN_CHUNK_USIZE]:
{
  pub array: [f32; C::VOXELS_IN_CHUNK_USIZE],
}

impl<C: ChunkSize> ChunkSampleArray<C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:
{
  #[inline]
  pub fn new(array: [f32; C::VOXELS_IN_CHUNK_USIZE]) -> Self {
    Self { array }
  }

  #[inline]
  pub fn new_with(default: f32) -> Self {
    Self::new([default; C::VOXELS_IN_CHUNK_USIZE])
  }

  #[inline]
  pub fn new_positive_zeroed() -> Self {
    Self::new_with(0.0)
  }

  #[inline]
  pub fn new_negative_zeroed() -> Self {
    Self::new_with(-0.0)
  }

  #[inline]
  fn index_xyz(x: u32, y: u32, z: u32) -> usize { (x + C::VOXELS_IN_CHUNK_ROW * y + C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW * z) as usize }

  #[inline]
  fn index(position: UVec3) -> usize { Self::index_xyz(position.x, position.y, position.z) }

  #[inline]
  pub fn sample(&self, position: UVec3) -> f32 {
    self.array[Self::index(position)]
  }

  #[inline]
  pub fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    &mut self.array[Self::index(position)]
  }

  #[inline]
  pub fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    self.array[Self::index_xyz(x, y, z)] = sample;
  }

  #[inline]
  pub fn set_all_to(&mut self, sample: f32) {
    for s in self.array.iter_mut() {
      *s = sample;
    }
  }

  #[inline]
  pub fn flip_all(&mut self) {
    for s in self.array.iter_mut() {
      *s *= -1.0;
    }
  }
}

// Chunk vertices

#[derive(Clone, Default, Debug)]
pub struct ChunkVertices {
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
}

impl ChunkVertices {
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
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.indices.clear();
  }
}

// LOD chunk vertices

#[derive(Clone, Default, Debug)]
pub struct LodChunkVertices {
  pub regular: ChunkVertices,
  pub transition_lo_x_chunk: ChunkVertices,
  pub transition_hi_x_chunk: ChunkVertices,
  pub transition_lo_y_chunk: ChunkVertices,
  pub transition_hi_y_chunk: ChunkVertices,
  pub transition_lo_z_chunk: ChunkVertices,
  pub transition_hi_z_chunk: ChunkVertices,
}

impl LodChunkVertices {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(
    regular: ChunkVertices,
    transition_lo_x_chunk: ChunkVertices,
    transition_hi_x_chunk: ChunkVertices,
    transition_lo_y_chunk: ChunkVertices,
    transition_hi_y_chunk: ChunkVertices,
    transition_lo_z_chunk: ChunkVertices,
    transition_hi_z_chunk: ChunkVertices,
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

