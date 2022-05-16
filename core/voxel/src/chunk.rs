use std::fmt::{Display, Formatter};
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{UVec3, Vec3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

// Chunk size

pub trait ChunkSize: Default + Copy + Clone + Send + 'static {
  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;

  const HALF_CELLS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const CELLS_IN_DECK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_DECK_USIZE: usize = Self::CELLS_IN_DECK as usize;

  const CELLS_IN_CHUNK_USIZE: usize = Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE;

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;

  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;

  type CellsChunkArray<T>: Sliceable<T>;
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T>;

  type VoxelsChunkArray<T>: Sliceable<T>;
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T>;

  type MarchingCubesSharedIndicesArray<T>: Sliceable<T>;
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T>;

  type TransvoxelSharedIndicesArray<T>: Sliceable<T>;
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T>;
}

pub trait Sliceable<T> {
  fn slice(&self) -> &[T];
  fn slice_mut(&mut self) -> &mut [T];
}

impl<T, const N: usize> Sliceable<T> for [T; N] {
  #[inline]
  fn slice(&self) -> &[T] { self }
  #[inline]
  fn slice_mut(&mut self) -> &mut [T] { self }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize1 {}

impl ChunkSize for ChunkSize1 {
  const CELLS_IN_CHUNK_ROW: u32 = 1;

  type CellsChunkArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize2 {}

impl ChunkSize for ChunkSize2 {
  const CELLS_IN_CHUNK_ROW: u32 = 2;

  type CellsChunkArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize16 {}

impl ChunkSize for ChunkSize16 {
  const CELLS_IN_CHUNK_ROW: u32 = 16;

  type CellsChunkArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}


// Chunk cell/voxel indices

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct CellIndex(u32);

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct VoxelIndex(u32);

pub trait ChunkIndices {
  fn cell_index_from_xyz(x: u32, y: u32, z: u32) -> CellIndex;

  #[inline]
  fn cell_index_from_uvec3(position: UVec3) -> CellIndex {
    Self::cell_index_from_xyz(position.x, position.y, position.z)
  }

  fn xyz_from_cell_index(cell_index: CellIndex) -> (u32, u32, u32);

  #[inline]
  fn uvec3_from_cell_index(cell_index: CellIndex) -> UVec3 {
    let (x, y, z) = Self::xyz_from_cell_index(cell_index);
    UVec3::new(x, y, z)
  }


  fn voxel_index_from_xyz(x: u32, y: u32, z: u32) -> VoxelIndex;

  #[inline]
  fn voxel_index_from_uvec3(position: UVec3) -> VoxelIndex {
    Self::voxel_index_from_xyz(position.x, position.y, position.z)
  }

  fn xyz_from_voxel_index(voxel_index: VoxelIndex) -> (u32, u32, u32);

  #[inline]
  fn uvec3_from_voxel_index(voxel_index: VoxelIndex) -> UVec3 {
    let (x, y, z) = Self::xyz_from_voxel_index(voxel_index);
    UVec3::new(x, y, z)
  }


  #[inline]
  fn cell_index_to_voxel_index(cell_index: CellIndex) -> VoxelIndex {
    let (x, y, z) = Self::xyz_from_cell_index(cell_index);
    Self::voxel_index_from_xyz(x, y, z)
  }

  #[inline]
  fn voxel_index_to_cell_index(voxel_index: VoxelIndex) -> CellIndex {
    let (x, y, z) = Self::xyz_from_voxel_index(voxel_index);
    Self::cell_index_from_xyz(x, y, z)
  }
}

#[inline]
pub const fn cell_index_from_xyz<C: ChunkSize>(x: u32, y: u32, z: u32) -> CellIndex {
  CellIndex(x + (C::CELLS_IN_CHUNK_ROW * y) + (C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW * z))
}

#[inline]
pub const fn voxel_index_from_xyz<C: ChunkSize>(x: u32, y: u32, z: u32) -> VoxelIndex {
  VoxelIndex(x + (C::VOXELS_IN_CHUNK_ROW * y) + (C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW * z))
}

impl<C: ChunkSize> ChunkIndices for C {
  #[inline]
  fn cell_index_from_xyz(x: u32, y: u32, z: u32) -> CellIndex {
    cell_index_from_xyz::<C>(x, y, z)
  }

  #[inline]
  fn xyz_from_cell_index(cell_index: CellIndex) -> (u32, u32, u32) {
    let mut i = cell_index.0;
    let z = i / (C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW);
    i -= z * (C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW);
    let y = i / C::CELLS_IN_CHUNK_ROW;
    let x = i % C::CELLS_IN_CHUNK_ROW;
    (x, y, z)
  }


  #[inline]
  fn voxel_index_from_xyz(x: u32, y: u32, z: u32) -> VoxelIndex {
    voxel_index_from_xyz::<C>(x, y, z)
  }

  #[inline]
  fn xyz_from_voxel_index(voxel_index: VoxelIndex) -> (u32, u32, u32) {
    let mut i = voxel_index.0;
    let z = i / (C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW);
    i -= z * (C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW);
    let y = i / C::VOXELS_IN_CHUNK_ROW;
    let x = i % C::VOXELS_IN_CHUNK_ROW;
    (x, y, z)
  }
}

impl CellIndex {
  #[inline]
  pub fn into_u32(self) -> u32 { self.0 }
  #[inline]
  pub fn into_usize(self) -> usize { self.0 as usize }
}

impl Into<u32> for CellIndex {
  #[inline]
  fn into(self) -> u32 { self.into_u32() }
}

impl Into<usize> for CellIndex {
  #[inline]
  fn into(self) -> usize { self.into_usize() }
}

impl std::ops::Add for CellIndex {
  type Output = CellIndex;
  #[inline]
  fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}

impl std::ops::Sub for CellIndex {
  type Output = CellIndex;
  #[inline]
  fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl VoxelIndex {
  #[inline]
  pub fn into_u32(self) -> u32 { self.0 }
  #[inline]
  pub fn into_usize(self) -> usize { self.0 as usize }
}

impl Into<u32> for VoxelIndex {
  #[inline]
  fn into(self) -> u32 { self.into_u32() }
}

impl Into<usize> for VoxelIndex {
  #[inline]
  fn into(self) -> usize { self.into_usize() }
}

impl std::ops::Add for VoxelIndex {
  type Output = VoxelIndex;
  #[inline]
  fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}

impl std::ops::Sub for VoxelIndex {
  type Output = VoxelIndex;
  #[inline]
  fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl Display for CellIndex {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl Display for VoxelIndex {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

// Chunk samples

pub enum ChunkSamples<C: ChunkSize> {
  /// All sampled values in the chunk are exactly `0.0`.
  Zero,
  /// All sampled values in the chunk are positive (i.e., `f32::is_sign_positive() == true`).
  Positive,
  /// All sampled values in the chunk are negative (i.e., `f32::is_sign_negative() == true`).
  Negative,
  /// Sampled values in the chunk are mixed.
  Mixed(ChunkSampleArray<C>),
}

impl<C: ChunkSize> ChunkSamples<C> {
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
pub struct ChunkSampleArray<C: ChunkSize> {
  pub array: C::VoxelsChunkArray<f32>,
}

impl<C: ChunkSize> ChunkSampleArray<C> {
  #[inline]
  pub fn new(array: C::VoxelsChunkArray<f32>) -> Self {
    Self { array }
  }

  #[inline]
  pub fn new_with(default: f32) -> Self {
    Self::new(C::create_voxel_chunk_array(default))
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
  pub fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    self.array.slice()[voxel_index.into_usize()]
  }

  #[inline]
  pub fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 {
    &mut self.array.slice_mut()[voxel_index.into_usize()]
  }

  #[inline]
  pub fn sample(&self, position: UVec3) -> f32 {
    self.sample_index(C::voxel_index_from_uvec3(position).into())
  }

  #[inline]
  pub fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    self.sample_index_mut(C::voxel_index_from_uvec3(position).into())
  }

  #[inline]
  pub fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    *self.sample_index_mut(C::voxel_index_from_xyz(x, y, z).into()) = sample;
  }

  #[inline]
  pub fn set_all_to(&mut self, sample: f32) {
    for s in self.array.slice_mut().iter_mut() {
      *s = sample;
    }
  }

  #[inline]
  pub fn flip_all(&mut self) {
    for s in self.array.slice_mut().iter_mut() {
      *s *= -1.0;
    }
  }
}

impl<C: ChunkSize> Default for ChunkSampleArray<C> {
  fn default() -> Self {
    Self::new_positive_zeroed()
  }
}


// Chunk mesh

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

