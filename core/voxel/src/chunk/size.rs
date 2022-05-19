// Chunk size trait

use crate::chunk::index::{CellIndex, VoxelIndex};

pub trait ChunkSize: Default + Copy + Clone + Send + 'static {
  // Cell constants

  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;

  const CELLS_IN_CHUNK_ROW_MINUS_ONE: u32 = Self::CELLS_IN_CHUNK_ROW - 1;
  const CELLS_IN_CHUNK_ROW_DIV_TWO: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const CELLS_IN_DECK: u32 = Self::CELLS_IN_CHUNK_ROW * Self::CELLS_IN_CHUNK_ROW;
  const CELLS_IN_DECK_USIZE: usize = Self::CELLS_IN_DECK as usize;

  const CELLS_IN_DECK_DOUBLE: u32 = Self::CELLS_IN_DECK * 2;
  const CELLS_IN_DECK_DOUBLE_USIZE: usize = Self::CELLS_IN_DECK_DOUBLE as usize;

  const CELLS_IN_CHUNK_USIZE: usize = Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE * Self::CELLS_IN_CHUNK_ROW_USIZE;

  const CELL_INDEX_UNIT_X: CellIndex = CellIndex::unit_x::<Self>();
  const CELL_INDEX_UNIT_Y: CellIndex = CellIndex::unit_y::<Self>();
  const CELL_INDEX_UNIT_Z: CellIndex = CellIndex::unit_z::<Self>();


  // Voxel constants

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;

  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;

  const VOXEL_INDEX_UNIT_X: VoxelIndex = VoxelIndex::unit_x::<Self>();
  const VOXEL_INDEX_UNIT_Y: VoxelIndex = VoxelIndex::unit_y::<Self>();
  const VOXEL_INDEX_UNIT_Z: VoxelIndex = VoxelIndex::unit_z::<Self>();


  // Array types

  type CellsChunkDeckDoubleArray<T: Copy>: Sliceable<T>;
  fn create_cell_chunk_deck_double_array<T: Copy>(default: T) -> Self::CellsChunkDeckDoubleArray<T>;

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

macro_rules! impl_chunk_size {
  ($n:literal, $id:ident) => {
    #[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub struct $id {}
    
    impl ChunkSize for $id {
      const CELLS_IN_CHUNK_ROW: u32 = $n;
    
      type CellsChunkDeckDoubleArray<T: Copy> = [T; Self::CELLS_IN_DECK_DOUBLE_USIZE];
      #[inline]
      fn create_cell_chunk_deck_double_array<T: Copy>(default: T) -> Self::CellsChunkDeckDoubleArray<T> { [default; Self::CELLS_IN_DECK_DOUBLE_USIZE] }
    
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
  };
}

impl_chunk_size!(1, ChunkSize1);
impl_chunk_size!(2, ChunkSize2);
impl_chunk_size!(16, ChunkSize16);
