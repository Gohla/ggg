use std::cmp::Ordering;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::size::ChunkSize;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Aabb32(u32);

impl Aabb32 {
  #[inline]
  pub fn root() -> Self { Self(1) }
  
  #[inline]
  pub fn depth(&self) -> u8 { ((32 - self.0.leading_zeros()) / 3) as u8 }
  #[inline]
  pub fn half_size(&self, max_half_size: u32) -> u32 { Self::half_size_internal(max_half_size, self.depth()) }
  #[inline]
  pub fn minimum_point(&self, max_half_size: u32) -> UVec3 {
    let depth = self.depth();
    let mut min = UVec3::zero();
    let mut code = self.0;
    let mut half_size = Self::half_size_internal(max_half_size, depth);
    for _ in (0..depth).rev() {
      let octant = code as u8 & 0b00000111;
      min += Self::octant_to_min(octant, half_size);
      code = code >> 3;
      half_size = half_size << 1;
    }
    min
  }

  #[inline]
  pub fn subdivide(&self) -> Aabb32Subdivide {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    Aabb32Subdivide::new(self)
  }
  #[inline]
  pub fn sibling_positive_x(&self) -> Option<Self> {
    let depth = self.depth();
    if depth == 0 { return None; }
    let first_x_bit = 1 << 0;
    if self.0 & first_x_bit != 0 { // Is X bit set on the first octant?
      let code = self.0 & !first_x_bit; // Unset X bit on the first octant.
      return Some(Self(code));
    }
    if depth == 1 { return None; } // Cannot go to parent.
    let second_x_bit = 1 << 3;
    if self.0 & second_x_bit != 0 { // Is X bit set on the second octant?
      let code = self.0 & !second_x_bit; // Unset X bit on the second octant.
      return Some(Self(code));
    }
    // There is a parent, but we cannot go to its positive x sibling as it is at the border.
    None
  }
  #[inline]
  pub fn sibling_positive_y(&self) -> Option<Self> {
    let depth = self.depth();
    if depth == 0 { return None; }
    let first_y_bit = 1 << 1;
    if self.0 & first_y_bit != 0 { // Is Y bit set on the first octant?
      let code = self.0 & !first_y_bit; // Unset Y bit on the first octant.
      return Some(Self(code));
    }
    if depth == 1 { return None; } // Cannot go to parent.
    let second_y_bit = 1 << 4;
    if self.0 & second_y_bit != 0 { // Is Y bit set on the second octant?
      let code = self.0 & !second_y_bit; // Unset Y bit on the second octant.
      return Some(Self(code));
    }
    // There is a parent, but we cannot go to its positive Y sibling as it is at the border.
    None
  }
  #[inline]
  pub fn sibling_positive_z(&self) -> Option<Self> {
    let depth = self.depth();
    if depth == 0 { return None; }
    let first_z_bit = 1 << 2;
    if self.0 & first_z_bit != 0 { // Is Z bit set on the first octant?
      let code = self.0 & !first_z_bit; // Unset Z bit on the first octant.
      return Some(Self(code));
    }
    if depth == 1 { return None; } // Cannot go to parent.
    let second_z_bit = 1 << 5;
    if self.0 & second_z_bit != 0 { // Is Z bit set on the second octant?
      let code = self.0 & !second_z_bit; // Unset Z bit on the second octant.
      return Some(Self(code));
    }
    // There is a parent, but we cannot go to its positive Z sibling as it is at the border.
    None
  }


  #[inline]
  fn half_size_internal(max_half_size: u32, depth: u8) -> u32 {
    debug_assert!(max_half_size.is_power_of_two(), "Max half size {} is not a power of 2", max_half_size);
    max_half_size >> depth // Right shift is divide by 2 for powers of 2.
  }
  #[inline]
  fn octant_to_min(octant: u8, half_size: u32) -> UVec3 {
    match octant {
      0 => UVec3::new(half_size, half_size, half_size),
      1 => UVec3::new(0, half_size, half_size),
      2 => UVec3::new(half_size, 0, half_size),
      3 => UVec3::new(0, 0, half_size),
      4 => UVec3::new(half_size, half_size, 0),
      5 => UVec3::new(0, half_size, 0),
      6 => UVec3::new(half_size, 0, 0),
      7 => UVec3::new(0, 0, 0),
      _ => unreachable!(),
    }
  }
}

pub struct Aabb32Subdivide {
  code: u32,
  octant: u8,
}

impl Aabb32Subdivide {
  #[inline]
  fn new(aabb: &Aabb32) -> Self { Self { code: aabb.0 << 3, octant: 0 } }
}

impl Iterator for Aabb32Subdivide {
  type Item = Aabb32;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.octant > 7 { return None; }
    let aabb = Aabb32(self.code | self.octant as u32);
    self.octant += 1;
    Some(aabb)
  }
}

#[cfg(test)]
mod tests {
  use ultraviolet::UVec3;

  use crate::lod::aabb::Aabb32;

  #[test]
  fn root() {
    let max_half_size = 2048;
    let root = Aabb32::root();
    assert_eq!(root.depth(), 0);
    assert_eq!(root.half_size(max_half_size), 2048);
    assert_eq!(root.minimum_point(max_half_size), UVec3::zero());
  }

  #[test]
  fn subdivide_once() {
    let max_half_size = 2048;
    let root = Aabb32::root();
    let subdivided: Vec<_> = root.subdivide().collect();
    {
      let sub = subdivided[0];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(1024, 1024, 1024));
    }
    {
      let sub = subdivided[1];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(0, 1024, 1024));
    }
    {
      let sub = subdivided[2];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(1024, 0, 1024));
    }
    {
      let sub = subdivided[3];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(0, 0, 1024));
    }
    {
      let sub = subdivided[4];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(1024, 1024, 0));
    }
    {
      let sub = subdivided[5];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(0, 1024, 0));
    }
    {
      let sub = subdivided[6];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(1024, 0, 0));
    }
    {
      let sub = subdivided[7];
      assert_eq!(sub.depth(), 1);
      assert_eq!(sub.half_size(max_half_size), 1024);
      assert_eq!(sub.minimum_point(max_half_size), UVec3::new(0, 0, 0));
    }
  }
}


/// Square axis-aligned bounding box, always in powers of 2, and with size always larger than 1.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AABB {
  pub min: UVec3,
  pub size: u32,
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
  pub fn step<C: ChunkSize>(&self) -> u32 { self.size / C::CELLS_IN_CHUNK_ROW }

  #[inline(always)]
  pub fn size_3d(&self) -> UVec3 { UVec3::new(self.size, self.size, self.size) }

  #[inline(always)]
  pub fn max_point(&self) -> UVec3 { self.min + self.size_3d() }

  #[inline]
  pub fn extends(&self) -> u32 {
    self.size / 2 // Note: no rounding needed because AABB is always size of 2 and > 1.
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
      Self::new_unchecked(cen, extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(min.x, min.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, min.z), extends),
      Self::new_unchecked(min, extends),
    ]
  }


  #[inline(always)]
  pub fn new_unchecked(min: UVec3, size: u32) -> Self {
    Self { min, size }
  }
}

impl PartialOrd<AABB> for AABB {
  fn partial_cmp(&self, other: &AABB) -> Option<Ordering> {
    (self.min.x, self.min.y, self.min.z, self.size).partial_cmp(&(other.min.x, other.min.y, other.min.z, other.size))
  }
}

impl Ord for AABB {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.min.x, self.min.y, self.min.z, self.size).cmp(&(other.min.x, other.min.y, other.min.z, other.size))
  }
}
