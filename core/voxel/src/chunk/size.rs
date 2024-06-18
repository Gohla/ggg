use crate::chunk::array::{Array, ConstArray};
use crate::chunk::index::{CellIndex, VoxelIndex};
use crate::chunk::shape::{ConstShape, Shape};
use crate::chunk::Value;

// Chunk size trait

pub trait ChunkSize: Default + Copy + Clone + Send + Sync + 'static {
  // Cell constants

  const CELLS_IN_CHUNK_ROW: u32;
  const CELLS_IN_CHUNK_ROW_F32: f32 = Self::CELLS_IN_CHUNK_ROW as f32;
  const CELLS_IN_CHUNK_ROW_USIZE: usize = Self::CELLS_IN_CHUNK_ROW as usize;

  const CELLS_IN_CHUNK_ROW_MINUS_ONE: u32 = Self::CELLS_IN_CHUNK_ROW - 1;
  const CELLS_IN_CHUNK_ROW_DIV_TWO: u32 = Self::CELLS_IN_CHUNK_ROW / 2;

  const CELLS_IN_ROW_QUAD: u32 = Self::CELLS_IN_CHUNK_ROW * 2 * 2;
  const CELLS_IN_ROW_QUAD_USIZE: usize = Self::CELLS_IN_ROW_QUAD as usize;

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

  type CellRowQuadXYShape: Shape<CellIndex>;
  type CellRowQuadYZShape: Shape<CellIndex>;
  type CellRowQuadXZShape: Shape<CellIndex>;
  type CellRowQuadArray<T: Value>: Array<T, CellIndex>;

  type CellDeckDoubleXShape: Shape<CellIndex>;
  type CellDeckDoubleYShape: Shape<CellIndex>;
  type CellDeckDoubleZShape: Shape<CellIndex>;
  type CellDeckDoubleArray<T: Value>: Array<T, CellIndex>;

  type CellChunkShape: Shape<CellIndex>;
  type CellChunkArray<T: Value>: Array<T, CellIndex>;

  type VoxelChunkShape: Shape<VoxelIndex>;
  type VoxelChunkArray<T: Value>: Array<T, VoxelIndex>;

  type MarchingCubesSharedIndicesShape: Shape<u32>;
  type MarchingCubesSharedIndicesArray<T: Value>: Array<T, u32>;

  type TransvoxelSharedIndicesShape: Shape<u32>;
  type TransvoxelSharedIndicesArray<T: Value>: Array<T, u32>;
}

// Chunk size implementations

macro_rules! impl_chunk_size {
  ($n:literal, $id:ident) => {
    #[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
    pub struct $id {}

    impl ChunkSize for $id {
      const CELLS_IN_CHUNK_ROW: u32 = $n;

      type CellRowQuadXYShape = ConstShape<CellIndex, 2, 2, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellRowQuadYZShape = ConstShape<CellIndex, {Self::CELLS_IN_CHUNK_ROW}, 2, 2>;
      type CellRowQuadXZShape = ConstShape<CellIndex, 2, {Self::CELLS_IN_CHUNK_ROW}, 2>;
      type CellRowQuadArray<T: Value> = ConstArray<T, CellIndex, {Self::CELLS_IN_ROW_QUAD_USIZE}>;

      type CellDeckDoubleXShape = ConstShape<CellIndex, 2, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellDeckDoubleYShape = ConstShape<CellIndex, {Self::CELLS_IN_CHUNK_ROW}, 2, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellDeckDoubleZShape = ConstShape<CellIndex, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}, 2>;
      type CellDeckDoubleArray<T: Value> = ConstArray<T, CellIndex, {Self::CELLS_IN_DECK_DOUBLE_USIZE}>;

      type CellChunkShape = ConstShape<CellIndex, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type CellChunkArray<T: Value> = ConstArray<T, CellIndex, {Self::CELLS_IN_CHUNK_USIZE}>;

      type VoxelChunkShape = ConstShape<VoxelIndex, {Self::VOXELS_IN_CHUNK_ROW}, {Self::VOXELS_IN_CHUNK_ROW}, {Self::VOXELS_IN_CHUNK_ROW}>;
      type VoxelChunkArray<T: Value> = ConstArray<T, VoxelIndex, {Self::VOXELS_IN_CHUNK_USIZE}>;

      type MarchingCubesSharedIndicesShape = ConstShape<u32, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type MarchingCubesSharedIndicesArray<T: Value> = ConstArray<T, u32, {Self::CELLS_IN_CHUNK_USIZE * 4}>;
      // TODO: this is specific to the X border? But currently not used because transvoxel does its own indexing.
      type TransvoxelSharedIndicesShape = ConstShape<u32, 2, {Self::CELLS_IN_CHUNK_ROW}, {Self::CELLS_IN_CHUNK_ROW}>;
      type TransvoxelSharedIndicesArray<T: Value> = ConstArray<T, u32, {Self::CELLS_IN_DECK_USIZE * 10}>;
    }
  };
}

impl_chunk_size!(1, ChunkSize1);
impl_chunk_size!(2, ChunkSize2);
impl_chunk_size!(6, ChunkSize6);
impl_chunk_size!(16, ChunkSize16);
impl_chunk_size!(32, ChunkSize32);
