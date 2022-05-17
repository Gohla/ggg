// Chunk size trait

pub trait ChunkSize: Default + Copy + Clone + Send + 'static {
  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;

  const HALF_CELLS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const CELLS_IN_CHUNK_BORDER: u32 = Self::CELLS_IN_CHUNK_ROW * 2;
  const CELLS_IN_CHUNK_BORDER_USIZE: usize = Self::CELLS_IN_CHUNK_BORDER as usize;

  const CELLS_IN_DECK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_DECK_USIZE: usize = Self::CELLS_IN_DECK as usize;

  const CELLS_IN_CHUNK_USIZE: usize = Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE;

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;

  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;

  type CellsChunkDeckArray<T: Copy>: Sliceable<T>;
  fn create_cell_chunk_deck_array<T: Copy>(default: T) -> Self::CellsChunkDeckArray<T>;

  type CellsChunkArray<T: Copy>: Sliceable<T>;
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T>;

  type VoxelsChunkArray<T: Copy>: Sliceable<T>;
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T>;

  type MarchingCubesSharedIndicesArray<T: Copy>: Sliceable<T>;
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T>;

  type TransvoxelSharedIndicesArray<T: Copy>: Sliceable<T>;
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T>;
}

// Sliceable (array) trait

pub trait Sliceable<T: Copy> {
  fn slice(&self) -> &[T];
  fn slice_mut(&mut self) -> &mut [T];
  fn index(&self, index: usize) -> T;
  fn index_ref(&self, index: usize) -> &T;
  fn index_mut(&mut self, index: usize) -> &mut T;
  fn len(&self) -> usize;
}

impl<T: Copy, const N: usize> Sliceable<T> for [T; N] {
  #[inline]
  fn slice(&self) -> &[T] { self }
  #[inline]
  fn slice_mut(&mut self) -> &mut [T] { self }
  #[inline]
  fn index(&self, index: usize) -> T { self[index] }
  #[inline]
  fn index_ref(&self, index: usize) -> &T { &self[index] }
  #[inline]
  fn index_mut(&mut self, index: usize) -> &mut T { &mut self[index] }
  #[inline]
  fn len(&self) -> usize { N }
}

// Chunk size implementations
// 1

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize1 {}

impl ChunkSize for ChunkSize1 {
  const CELLS_IN_CHUNK_ROW: u32 = 1;

  type CellsChunkDeckArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE];
  #[inline]
  fn create_cell_chunk_deck_array<T: Copy>(default: T) -> Self::CellsChunkDeckArray<T> { [default; Self::CELLS_IN_DECK_USIZE] }
  
  type CellsChunkArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T: Copy> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}

// 2

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize2 {}

impl ChunkSize for ChunkSize2 {
  const CELLS_IN_CHUNK_ROW: u32 = 2;

  type CellsChunkDeckArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE];
  #[inline]
  fn create_cell_chunk_deck_array<T: Copy>(default: T) -> Self::CellsChunkDeckArray<T> { [default; Self::CELLS_IN_DECK_USIZE] }
  
  type CellsChunkArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T: Copy> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}

// 16

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkSize16 {}

impl ChunkSize for ChunkSize16 {
  const CELLS_IN_CHUNK_ROW: u32 = 16;

  type CellsChunkDeckArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE];
  #[inline]
  fn create_cell_chunk_deck_array<T: Copy>(default: T) -> Self::CellsChunkDeckArray<T> { [default; Self::CELLS_IN_DECK_USIZE] }

  type CellsChunkArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE];
  #[inline]
  fn create_cell_chunk_array<T: Copy>(default: T) -> Self::CellsChunkArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE] }

  type VoxelsChunkArray<T: Copy> = [T; Self::VOXELS_IN_CHUNK_USIZE];
  #[inline]
  fn create_voxel_chunk_array<T: Copy>(default: T) -> Self::VoxelsChunkArray<T> { [default; Self::VOXELS_IN_CHUNK_USIZE] }

  type MarchingCubesSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_CHUNK_USIZE * 4];
  #[inline]
  fn create_marching_cubes_shared_indices_array<T: Copy>(default: T) -> Self::MarchingCubesSharedIndicesArray<T> { [default; Self::CELLS_IN_CHUNK_USIZE * 4] }

  type TransvoxelSharedIndicesArray<T: Copy> = [T; Self::CELLS_IN_DECK_USIZE * 10];
  #[inline]
  fn create_transvoxel_shared_indices_array<T: Copy>(default: T) -> Self::TransvoxelSharedIndicesArray<T> { [default; Self::CELLS_IN_DECK_USIZE * 10] }
}
