use std::fmt::{Display, Formatter};

use ultraviolet::UVec3;

use crate::chunk::size::ChunkSize;

// Index wrapper structs

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct CellIndex(u32);

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct VoxelIndex(u32);

// Indices trait

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

// Indices implementation

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

// Const functions (cannot be in trait)

#[inline]
pub const fn cell_index_from_xyz<C: ChunkSize>(x: u32, y: u32, z: u32) -> CellIndex {
  CellIndex(x + (C::CELLS_IN_CHUNK_ROW * y) + (C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW * z))
}

#[inline]
pub const fn voxel_index_from_xyz<C: ChunkSize>(x: u32, y: u32, z: u32) -> VoxelIndex {
  VoxelIndex(x + (C::VOXELS_IN_CHUNK_ROW * y) + (C::VOXELS_IN_CHUNK_ROW * C::VOXELS_IN_CHUNK_ROW * z))
}

// Index conversion

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


// Operators

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


// Display

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
