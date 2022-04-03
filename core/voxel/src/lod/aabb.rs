use ultraviolet::{UVec3, Vec3};

use crate::chunk::ChunkSize;

/// Square axis-aligned bounding box, always in powers of 2, and with size always larger than 1.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AABB {
  min: UVec3,
  size: u32,
}

impl AABB {
  #[inline]
  pub fn from_size(size: u32) -> Self {
    assert_ne!(size, 0, "Size may not be 0");
    assert_ne!(size, 1, "Size may not be 1");
    assert!(size.is_power_of_two(), "Size {} must be a power of 2", size);
    let min = UVec3::new(0, 0, 0);
    Self { min, size }
  }

  #[inline(always)]
  pub fn min(&self) -> UVec3 { self.min }

  #[inline(always)]
  pub fn size(&self) -> u32 { self.size }

  #[inline(always)]
  pub fn step<C: ChunkSize>(&self) -> u32 { self.size() / C::CELLS_IN_CHUNK_ROW }

  #[inline(always)]
  pub fn size_3d(&self) -> UVec3 { UVec3::new(self.size, self.size, self.size) }

  #[inline(always)]
  pub fn max(&self) -> UVec3 { self.min + self.size_3d() }

  #[inline]
  pub fn extends(&self) -> u32 {
    self.size() / 2 // Note: no rounding needed because AABB is always size of 2 and > 1.
  }

  #[inline]
  pub fn extends_3d(&self) -> UVec3 {
    let extends = self.extends();
    UVec3::new(extends, extends, extends)
  }

  #[inline]
  pub fn center(&self) -> UVec3 {
    self.min + self.extends_3d()
  }

  #[inline]
  pub fn distance_from(&self, point: Vec3) -> f32 {
    // TODO: copied from voxel-planets, check if this is correct and efficient?
    let distance_to_center = (point - self.center().into()).abs();
    let extends = self.extends_3d().into();
    let v = Vec3::zero().max_by_component(distance_to_center - extends).map(|f| f.powf(2.0));
    let distance = (v.x + v.y + v.z).sqrt();
    distance
  }

  #[inline]
  pub fn subdivide(&self) -> [AABB; 8] {
    let min = self.min;
    let cen = self.center();
    let extends = self.extends();
    [
      Self::new_unchecked(min, extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, min.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(min.x, min.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, cen.z), extends),
      Self::new_unchecked(cen, extends),
    ]
  }


  #[inline(always)]
  fn new_unchecked(min: UVec3, size: u32) -> Self {
    Self { min, size }
  }
}
