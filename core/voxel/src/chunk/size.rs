use crate::chunk::array::{Array, ConstArray};
use crate::chunk::index::{CellIndex, VoxelIndex};
use crate::chunk::shape::{ConstShape, Shape};

// Chunk size trait

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


  // Voxel constants

  const VOXELS_IN_CHUNK_ROW: u32 = Self::CELLS_IN_CHUNK_ROW + 1;
  const VOXELS_IN_CHUNK_ROW_USIZE: usize = Self::VOXELS_IN_CHUNK_ROW as usize;

  const VOXELS_IN_CHUNK: u32 = Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW * Self::VOXELS_IN_CHUNK_ROW;
  const VOXELS_IN_CHUNK_USIZE: usize = Self::VOXELS_IN_CHUNK as usize;


  // Array types

  type CellDeckDoubleShape: Shape<CellIndex>;
  type CellDeckDoubleArray<T: Copy>: Array<T, CellIndex>;
  type CellChunkShape: Shape<CellIndex>;
  type CellChunkArray<T: Copy>: Array<T, CellIndex>;

  type VoxelChunkShape: Shape<VoxelIndex>;
  type VoxelChunkArray<T: Copy>: Array<T, VoxelIndex>;

  type MarchingCubesSharedIndicesShape: Shape<u32>;
  type MarchingCubesSharedIndicesArray<T: Copy>: Array<T, u32>;
  type TransvoxelSharedIndicesShape: Shape<u32>;
  type TransvoxelSharedIndicesArray<T: Copy>: Array<T, u32>;
}

// Chunk size implementations

macro_rules! impl_chunk_size {
  ($n:literal, $id:ident) => {
    #[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub struct $id {}
    
    impl ChunkSize for $id {
      const CELLS_IN_CHUNK_ROW: u32 = $n;
    
      // TODO: this is specific to surface nets X border
      type CellDeckDoubleShape = ConstShape<CellIndex, 2, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellDeckDoubleArray<T: Copy> = ConstArray<T, CellIndex, {Self::CELLS_IN_DECK_DOUBLE_USIZE}>;
      type CellChunkShape = ConstShape<CellIndex, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellChunkArray<T: Copy> = ConstArray<T, CellIndex, {Self::CELLS_IN_CHUNK_USIZE}>;
      
      type VoxelChunkShape = ConstShape<VoxelIndex, {Self::VOXELS_IN_CHUNK_ROW}, {Self::VOXELS_IN_CHUNK_ROW}, {Self::VOXELS_IN_CHUNK_ROW}>;
      type VoxelChunkArray<T: Copy> = ConstArray<T, VoxelIndex, {Self::VOXELS_IN_CHUNK_USIZE}>;
      
      type MarchingCubesSharedIndicesShape = ConstShape<u32, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type MarchingCubesSharedIndicesArray<T: Copy> = ConstArray<T, u32, {Self::CELLS_IN_CHUNK_USIZE * 4}>;
      // TODO: this is specific to the X border? But currently not used because transvoxel does its own indexing.
      type TransvoxelSharedIndicesShape = ConstShape<u32, 2, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type TransvoxelSharedIndicesArray<T: Copy> = ConstArray<T, u32, {Self::CELLS_IN_DECK_USIZE * 10}>;
    }
  };
}

impl_chunk_size!(1, ChunkSize1);
impl_chunk_size!(2, ChunkSize2);
impl_chunk_size!(16, ChunkSize16);
