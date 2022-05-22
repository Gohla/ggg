use std::fmt::{Display, Formatter};

use ultraviolet::UVec3;

use crate::chunk::shape::{index_from_xyz, Shape};

// Index trait

pub trait Index: Copy {
  fn from_u32(i: u32) -> Self;
  fn into_u32(self) -> u32;
  fn into_usize(self) -> usize;
}


// u32 implementation

impl Index for u32 {
  #[inline]
  fn from_u32(i: u32) -> Self { i }
  #[inline]
  fn into_u32(self) -> u32 { self }
  #[inline]
  fn into_usize(self) -> usize { self as usize }
}


// Cell index implementations

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct CellIndex(u32);

impl Index for CellIndex {
  #[inline]
  fn from_u32(i: u32) -> Self { Self(i) }
  #[inline]
  fn into_u32(self) -> u32 { self.0 }
  #[inline]
  fn into_usize(self) -> usize { self.0 as usize }
}

impl CellIndex {
  #[inline]
  pub(crate) const fn from_xyz<S: Shape<Self>>(x: u32, y: u32, z: u32) -> Self { Self(index_from_xyz::<Self, S>(x, y, z)) }

  #[inline]
  pub(crate) const fn unit_x<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(1, 0, 0) }
  #[inline]
  pub(crate) const fn unit_y<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(0, 1, 0) }
  #[inline]
  pub(crate) const fn unit_z<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(0, 0, 1) }

  #[inline]
  pub fn to_xyz<S: Shape<Self>>(self) -> (u32, u32, u32) { S::index_into_xyz(self) }
  #[inline]
  pub fn to_pos<S: Shape<Self>>(self) -> UVec3 { S::index_into_pos(self) }
  #[inline]
  pub fn to_min_voxel_index<SC: Shape<Self>, SV: Shape<VoxelIndex>>(self) -> VoxelIndex {
    let (x, y, z) = self.to_xyz::<SC>();
    SV::index_from_xyz(x, y, z)
  }
}

impl Into<u32> for CellIndex {
  #[inline]
  fn into(self) -> u32 { self.0 }
}

impl Into<usize> for CellIndex {
  #[inline]
  fn into(self) -> usize { self.0 as usize }
}

impl From<u32> for CellIndex {
  #[inline]
  fn from(i: u32) -> Self { Self(i) }
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

impl Display for CellIndex {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}


// Voxel index implementation

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct VoxelIndex(u32);

impl Index for VoxelIndex {
  #[inline]
  fn from_u32(i: u32) -> Self { Self(i) }
  #[inline]
  fn into_u32(self) -> u32 { self.0 }
  #[inline]
  fn into_usize(self) -> usize { self.0 as usize }
}

impl VoxelIndex {
  #[inline]
  pub(crate) const fn from_xyz<S: Shape<Self>>(x: u32, y: u32, z: u32) -> Self { Self(index_from_xyz::<Self, S>(x, y, z)) }

  #[inline]
  pub(crate) const fn unit_x<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(1, 0, 0) }
  #[inline]
  pub(crate) const fn unit_y<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(0, 1, 0) }
  #[inline]
  pub(crate) const fn unit_z<S: Shape<Self>>() -> Self { Self::from_xyz::<S>(0, 0, 1) }

  #[inline]
  pub fn to_xyz<S: Shape<Self>>(self) -> (u32, u32, u32) { S::index_into_xyz(self) }
  #[inline]
  pub fn to_pos<S: Shape<Self>>(self) -> UVec3 { S::index_into_pos(self) }
  #[inline]
  pub fn to_max_cell_index<SV: Shape<Self>, SC: Shape<CellIndex>>(self) -> CellIndex {
    let (x, y, z) = self.to_xyz::<SV>();
    SC::index_from_xyz(x, y, z)
  }
}

impl Into<u32> for VoxelIndex {
  #[inline]
  fn into(self) -> u32 { self.0 }
}

impl Into<usize> for VoxelIndex {
  #[inline]
  fn into(self) -> usize { self.0 as usize }
}

impl From<u32> for VoxelIndex {
  #[inline]
  fn from(i: u32) -> Self { Self(i) }
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

impl Display for VoxelIndex {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}
