use std::num::NonZeroU32;
use std::ops::Index;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::size::ChunkSize;

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Aabb(NonZeroU32);

impl Aabb {
  #[inline]
  pub fn root() -> Self { Self(unsafe { NonZeroU32::new_unchecked(2) }) }

  #[inline]
  pub fn depth(&self) -> u8 { ((31 - self.0.leading_zeros()) / 3) as u8 }

  #[inline]
  pub fn size(&self, root_size: u32) -> u32 { Self::size_internal(root_size, self.depth()) }
  #[inline]
  pub fn half_size(&self, root_size: u32) -> u32 { Self::half_size_internal(root_size, self.depth()) }
  #[inline]
  pub fn minimum_point(&self, root_size: u32) -> UVec3 {
    let depth = self.depth();
    let size = Self::size_internal(root_size, depth);
    Self::minimum_point_internal(depth, size, self.0.get())
  }
  #[inline]
  pub fn center_point(&self, root_size: u32) -> UVec3 {
    let minimum_point = self.minimum_point(root_size);
    let half_size = self.half_size(root_size);
    Self::center_point_internal(minimum_point, half_size)
  }
  #[inline]
  pub fn maximum_point(&self, root_size: u32) -> UVec3 {
    let minimum_point = self.minimum_point(root_size);
    let size = self.size(root_size);
    Self::maximum_point_internal(minimum_point, size)
  }
  #[inline]
  pub fn step<C: ChunkSize>(&self, root_size: u32) -> u32 { self.size(root_size) / C::CELLS_IN_CHUNK_ROW }
  #[inline]
  pub fn closest_point(&self, root_size: u32, mut point: Vec3) -> Vec3 {
    let depth = self.depth();
    let size = Self::size_internal(root_size, depth);
    let minimum_point = Self::minimum_point_internal(depth, size, self.0.get());
    let maximum_point = Self::maximum_point_internal(minimum_point, size);
    point.clamp(minimum_point.into(), maximum_point.into());
    point
  }
  #[inline]
  pub fn distance_from(&self, root_size: u32, point: Vec3) -> f32 {
    let closest_point = self.closest_point(root_size, point);
    (closest_point - point).mag()
  }
  #[inline]
  pub fn with_size(&self, root_size: u32) -> AabbWithSize {
    AabbWithSize { root_size, inner: *self }
  }

  #[inline]
  pub fn subdivide(&self) -> AabbSubdivide {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    let code = self.child_code();
    unsafe {
      AabbSubdivide {
        base: Self::new_unchecked(code | 0b000_0),
        x: Self::new_unchecked(code | 0b001_0),
        y: Self::new_unchecked(code | 0b010_0),
        xy: Self::new_unchecked(code | 0b011_0),
        z: Self::new_unchecked(code | 0b100_0),
        xz: Self::new_unchecked(code | 0b101_0),
        yz: Self::new_unchecked(code | 0b110_0),
        xyz: Self::new_unchecked(code | 0b111_0),
      }
    }
  }
  #[inline]
  pub fn subdivide_iter(&self) -> AabbSubdivideIter {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    AabbSubdivideIter::new(self)
  }
  #[inline]
  pub fn subdivide_array(&self) -> [Aabb; 8] {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    let code = self.child_code();
    unsafe {
      [
        Self::new_unchecked(code | 0b000_0),
        Self::new_unchecked(code | 0b001_0),
        Self::new_unchecked(code | 0b010_0),
        Self::new_unchecked(code | 0b011_0),
        Self::new_unchecked(code | 0b100_0),
        Self::new_unchecked(code | 0b101_0),
        Self::new_unchecked(code | 0b110_0),
        Self::new_unchecked(code | 0b111_0),
      ]
    }
  }

  #[inline]
  pub fn sibling_positive_x(&self) -> Option<Self> { self.positive_sibling::<0>() }
  #[inline]
  pub fn sibling_positive_y(&self) -> Option<Self> { self.positive_sibling::<1>() }
  #[inline]
  pub fn sibling_positive_z(&self) -> Option<Self> { self.positive_sibling::<2>() }
  #[inline]
  pub fn sibling_positive_xy(&self) -> Option<Self> { self.sibling_positive_x().and_then(|aabb| aabb.sibling_positive_y()) }
  #[inline]
  pub fn sibling_positive_yz(&self) -> Option<Self> { self.sibling_positive_y().and_then(|aabb| aabb.sibling_positive_z()) }
  #[inline]
  pub fn sibling_positive_xz(&self) -> Option<Self> { self.sibling_positive_x().and_then(|aabb| aabb.sibling_positive_z()) }
  #[inline]
  fn positive_sibling<const O: u8>(&self) -> Option<Self> {
    let depth = self.depth();
    let mut code = self.0.get();
    for d in 0..depth {
      let bit = 1 << ((d * 3) + O + 1); // + 1 to skip user bit
      if (code & bit) == 0 { // If bit is unset, set it to go to the positive sibling; and we're done.
        unsafe { return Some(Self::new_unchecked(code | bit)); }
      } else { // Otherwise unset the bit to go to the negative sibling and continue.
        code = code & !bit;
      }
    }
    None // No parent was found with the bit set, so we couldn't go to a positive sibling anywhere.
  }

  #[inline]
  pub fn is_user_bit_set(&self) -> bool { self.0.get() & 1 != 0 }
  #[inline]
  pub fn set_user_bit(&mut self) { unsafe { self.update_unchecked(self.0.get() | 1) } }
  #[inline]
  pub fn unset_user_bit(&mut self) { unsafe { self.update_unchecked(self.0.get() & (!1)) } }
  #[inline]
  pub fn with_user_bit_set(&self) -> Self { unsafe { Self::new_unchecked(self.0.get() | 1) } }
  #[inline]
  pub fn with_user_bit_unset(&self) -> Self { unsafe { Self::new_unchecked(self.0.get() & (!1)) } }

  #[inline]
  unsafe fn new_unchecked(code: u32) -> Self {
    debug_assert!(code != 0);
    Self(NonZeroU32::new_unchecked(code))
  }
  #[inline]
  unsafe fn update_unchecked(&mut self, code: u32) {
    debug_assert!(code != 0);
    self.0 = NonZeroU32::new_unchecked(code);
  }
  #[inline]
  fn child_code(&self) -> u32 {
    let code = self.0.get();
    let user_bit_set = code & 1 != 0;
    let code = code & (!1); // Unset user bit to prevent it from shifting.
    let code = code << 3;
    code | user_bit_set as u32 // Set the user bit again if it was set.
  }

  #[inline]
  fn size_internal(root_size: u32, depth: u8) -> u32 {
    debug_assert!(root_size.is_power_of_two(), "Root size {} is not a power of 2", root_size);
    root_size >> depth // Right shift is divide by 2 for powers of 2.
  }
  #[inline]
  fn half_size_internal(root_size: u32, depth: u8) -> u32 {
    debug_assert!(root_size.is_power_of_two(), "Root size {} is not a power of 2", root_size);
    root_size >> depth + 1 // Right shift is divide by 2 for powers of 2.
  }

  #[inline]
  fn minimum_point_internal(depth: u8, mut size: u32, mut code: u32) -> UVec3 {
    let mut minimum_point = UVec3::zero();
    for _ in 0..depth {
      let octant = code as u8 & 0b111_0;
      minimum_point += Self::octant_to_minimum_point(octant, size);
      code = code >> 3;
      size = size << 1;
    }
    minimum_point
  }
  #[inline]
  fn center_point_internal(minimum_point: UVec3, half_size: u32) -> UVec3 {
    minimum_point + UVec3::broadcast(half_size)
  }
  #[inline]
  fn maximum_point_internal(minimum_point: UVec3, size: u32) -> UVec3 {
    minimum_point + UVec3::broadcast(size)
  }

  #[inline]
  fn octant_to_minimum_point(octant: u8, size: u32) -> UVec3 {
    match octant {
      0b000_0 => UVec3::new(0, 0, 0),
      0b001_0 => UVec3::new(size, 0, 0),
      0b010_0 => UVec3::new(0, size, 0),
      0b011_0 => UVec3::new(size, size, 0),
      0b100_0 => UVec3::new(0, 0, size),
      0b101_0 => UVec3::new(size, 0, size),
      0b110_0 => UVec3::new(0, size, size),
      0b111_0 => UVec3::new(size, size, size),
      _ => unreachable!(),
    }
  }
}


// Sized AABB

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AabbWithSize {
  pub root_size: u32,
  pub inner: Aabb,
}

impl AabbWithSize {
  #[inline]
  pub fn depth(&self) -> u8 { self.inner.depth() }

  #[inline]
  pub fn half_size(&self) -> u32 { self.inner.half_size(self.root_size) }
  #[inline]
  pub fn size(&self) -> u32 { self.inner.size(self.root_size) }
  #[inline]
  pub fn minimum_point(&self) -> UVec3 { self.inner.minimum_point(self.root_size) }
  #[inline]
  pub fn center_point(&self) -> UVec3 { self.inner.center_point(self.root_size) }
  #[inline]
  pub fn maximum_point(&self) -> UVec3 { self.inner.maximum_point(self.root_size) }
  #[inline]
  pub fn step<C: ChunkSize>(&self) -> u32 { self.inner.step::<C>(self.root_size) }
  #[inline]
  pub fn closest_point(&self, point: Vec3) -> Vec3 { self.inner.closest_point(self.root_size, point) }
  #[inline]
  pub fn distance_from(&self, point: Vec3) -> f32 { self.inner.distance_from(self.root_size, point) }

  #[inline]
  pub fn subdivide(&self) -> AabbSubdivideIter { self.inner.subdivide_iter() }
  #[inline]
  pub fn subdivide_array(&self) -> [Aabb; 8] { self.inner.subdivide_array() }

  #[inline]
  pub fn sibling_positive_x(&self) -> Option<Self> { self.inner.sibling_positive_x().map(|inner| self.wrap(inner)) }
  #[inline]
  pub fn sibling_positive_y(&self) -> Option<Self> { self.inner.sibling_positive_y().map(|inner| self.wrap(inner)) }
  #[inline]
  pub fn sibling_positive_z(&self) -> Option<Self> { self.inner.sibling_positive_z().map(|inner| self.wrap(inner)) }
  #[inline]
  pub fn sibling_positive_xy(&self) -> Option<Self> { self.inner.sibling_positive_xy().map(|inner| self.wrap(inner)) }
  #[inline]
  pub fn sibling_positive_yz(&self) -> Option<Self> { self.inner.sibling_positive_yz().map(|inner| self.wrap(inner)) }
  #[inline]
  pub fn sibling_positive_xz(&self) -> Option<Self> { self.inner.sibling_positive_xz().map(|inner| self.wrap(inner)) }

  #[inline]
  pub fn is_user_bit_set(&self) -> bool { self.inner.is_user_bit_set() }
  #[inline]
  pub fn set_user_bit(&mut self) { self.inner.set_user_bit() }
  #[inline]
  pub fn unset_user_bit(&mut self) { self.inner.unset_user_bit() }
  #[inline]
  pub fn with_user_bit_set(&self) -> Self { self.wrap(self.inner.with_user_bit_set()) }
  #[inline]
  pub fn with_user_bit_unset(&self) -> Self { self.wrap(self.inner.with_user_bit_unset()) }

  #[inline]
  fn wrap(&self, inner: Aabb) -> Self { Self { root_size: self.root_size, inner } }
}


// Per AABB subdivide struct

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PerAabbSubdivide<T> {
  pub base: T,
  pub x: T,
  pub y: T,
  pub xy: T,
  pub z: T,
  pub xz: T,
  pub yz: T,
  pub xyz: T,
}

impl<T> PerAabbSubdivide<T> {
  #[inline]
  pub fn with(default: T) -> Self where T: Copy {
    Self { base: default, x: default, y: default, xy: default, z: default, xz: default, yz: default, xyz: default, }
  }

  #[inline]
  pub fn with_default() -> Self where T: Default {
    Self { base: T::default(), x: T::default(), y: T::default(), xy: T::default(), z: T::default(), xz: T::default(), yz: T::default(), xyz: T::default(), }
  }

  #[inline]
  pub fn from_array(array: [T; 8]) -> Self {
    let [base, x, y, xy, z, xz, yz, xyz] = array;
    Self { base, x, y, xy, z, xz, yz, xyz, }
  }

  #[inline]
  pub fn into_array(self) -> [T; 8] {
    [
      self.base,
      self.x,
      self.y,
      self.xy,
      self.z,
      self.xz,
      self.yz,
      self.xyz,
    ]
  }

  #[inline]
  pub fn into_iter(self) -> PerAabbSubdivideIter<T> where T: Copy {
    PerAabbSubdivideIter::new(self)
  }
}

impl<T> From<PerAabbSubdivide<T>> for [T; 8] {
  #[inline]
  fn from(sub: PerAabbSubdivide<T>) -> Self { sub.into_array() }
}

impl<T> From<[T; 8]> for PerAabbSubdivide<T> {
  #[inline]
  fn from(array: [T; 8]) -> Self { PerAabbSubdivide::from_array(array) }
}

impl<T> Index<usize> for PerAabbSubdivide<T> {
  type Output = T;

  #[inline]
  fn index(&self, index: usize) -> &Self::Output {
    match index {
      0 => &self.base,
      1 => &self.x,
      2 => &self.y,
      3 => &self.xy,
      4 => &self.z,
      5 => &self.xz,
      6 => &self.yz,
      7 => &self.xyz,
      _ => unreachable!(),
    }
  }
}

impl<T> Index<u8> for PerAabbSubdivide<T> {
  type Output = T;

  #[inline]
  fn index(&self, index: u8) -> &Self::Output {
    match index {
      0 => &self.base,
      1 => &self.x,
      2 => &self.y,
      3 => &self.xy,
      4 => &self.z,
      5 => &self.xz,
      6 => &self.yz,
      7 => &self.xyz,
      _ => unreachable!(),
    }
  }
}

impl<T> IntoIterator for PerAabbSubdivide<T> where T: Copy {
  type Item = T;
  type IntoIter = PerAabbSubdivideIter<T>;
  #[inline]
  fn into_iter(self) -> Self::IntoIter { self.into_iter() }
}

pub struct PerAabbSubdivideIter<T> {
  sub: PerAabbSubdivide<T>,
  i: u8,
}

impl<T> PerAabbSubdivideIter<T> {
  #[inline]
  fn new(sub: PerAabbSubdivide<T>) -> Self { Self { sub, i: 0 } }
}

impl<T> Iterator for PerAabbSubdivideIter<T> where T: Copy {
  type Item = T;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.i > 7 { return None; }
    let v = self.sub[self.i];
    self.i += 1;
    Some(v)
  }
}


// AABB subdivide struct

pub type AabbSubdivide = PerAabbSubdivide<Aabb>;


// Subdivide iterator

pub struct AabbSubdivideIter {
  code: u32,
  octant: u8,
}

impl AabbSubdivideIter {
  #[inline]
  fn new(aabb: &Aabb) -> Self { Self { code: aabb.child_code(), octant: 0 } }
}

impl Iterator for AabbSubdivideIter {
  type Item = Aabb;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.octant > 7 { return None; }
    let aabb = unsafe { Aabb::new_unchecked(self.code | (self.octant as u32) << 1) };
    self.octant += 1;
    Some(aabb)
  }
}


// Tests

#[cfg(test)]
mod tests {
  use std::mem::size_of;

  use ultraviolet::{UVec3, Vec3};

  use crate::lod::aabb::{Aabb, AabbSubdivide, AabbWithSize};

  #[test]
  fn root() {
    let root_size = 4096;
    let root_half_size = 2048;
    let root = Aabb::root().with_size(root_size);
    assert_eq!(0, root.depth());
    assert_eq!(root_size, root.size());
    assert_eq!(root_half_size, root.half_size());
    assert_eq!(UVec3::zero(), root.minimum_point());
    assert_eq!(UVec3::broadcast(root_half_size), root.center_point());
    assert_eq!(UVec3::broadcast(root_size), root.maximum_point());
    assert_eq!(None, root.sibling_positive_x());
    assert_eq!(None, root.sibling_positive_y());
    assert_eq!(None, root.sibling_positive_z());
    assert_eq!(None, root.sibling_positive_xy());
    assert_eq!(None, root.sibling_positive_yz());
    assert_eq!(None, root.sibling_positive_xz());
    assert_eq!(false, root.is_user_bit_set());
  }

  fn test_subdivided(root_size: u32, depth: u8, size: u32, offset: UVec3, user_bit_set: bool, subdivided: AabbSubdivide) {
    let half_size = size / 2;
    { // Base
      let sub = subdivided.base.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, 0, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(subdivided.x), sub.inner.sibling_positive_x());
        assert_eq!(Some(subdivided.y), sub.inner.sibling_positive_y());
        assert_eq!(Some(subdivided.z), sub.inner.sibling_positive_z());
        assert_eq!(Some(subdivided.xy), sub.inner.sibling_positive_xy());
        assert_eq!(Some(subdivided.yz), sub.inner.sibling_positive_yz());
        assert_eq!(Some(subdivided.xz), sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // X
      let sub = subdivided.x.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, 0, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.inner.sibling_positive_x());
        assert_eq!(Some(subdivided.xy), sub.inner.sibling_positive_y());
        assert_eq!(Some(subdivided.xz), sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // Y
      let sub = subdivided.y.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, size, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, size + half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size * 2, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(subdivided.xy), sub.inner.sibling_positive_x());
        assert_eq!(None, sub.inner.sibling_positive_y());
        assert_eq!(Some(subdivided.yz), sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // XY
      let sub = subdivided.xy.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, size, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, size + half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size * 2, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.inner.sibling_positive_x());
        assert_eq!(None, sub.inner.sibling_positive_y());
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // Z
      let sub = subdivided.z.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, 0, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(subdivided.xz), sub.inner.sibling_positive_x());
        assert_eq!(Some(subdivided.yz), sub.inner.sibling_positive_y());
        assert_eq!(None, sub.inner.sibling_positive_z());
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // XZ 
      let sub = subdivided.xz.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, 0, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.inner.sibling_positive_x());
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_y());
        assert_eq!(None, sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // YZ
      let sub = subdivided.yz.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, size, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, size + half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size * 2, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(subdivided.xyz), sub.inner.sibling_positive_x());
        assert_eq!(None, sub.inner.sibling_positive_y());
        assert_eq!(None, sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
    { // XYZ
      let sub = subdivided.xyz.with_size(root_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, size, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, size + half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size * 2, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.inner.sibling_positive_x());
        assert_eq!(None, sub.inner.sibling_positive_y());
        assert_eq!(None, sub.inner.sibling_positive_z());
        assert_eq!(None, sub.inner.sibling_positive_xy());
        assert_eq!(None, sub.inner.sibling_positive_yz());
        assert_eq!(None, sub.inner.sibling_positive_xz());
      }
      assert_eq!(user_bit_set, sub.is_user_bit_set());
    }
  }

  #[test]
  fn subdivide_once() {
    let root_size = 4096;
    test_subdivided(root_size, 1, root_size / 2, UVec3::zero(), false, Aabb::root().subdivide());
  }

  #[test]
  fn subdivide_twice() {
    let root_size = 4096;
    for sub in Aabb::root().subdivide_array() {
      test_subdivided(root_size, 2, root_size / 4, sub.minimum_point(root_size), false, sub.subdivide());
    }
  }

  #[test]
  fn subdivide_trice() {
    let root_size = 4096;
    for sub_1 in Aabb::root().subdivide_array() {
      for sub_2 in sub_1.subdivide_array() {
        test_subdivided(root_size, 3, root_size / 8, sub_2.minimum_point(root_size), false, sub_2.subdivide());
      }
    }
  }

  #[test]
  fn subdivide_struct() {
    let root = Aabb::root();
    assert_eq!(root.subdivide_array(), root.subdivide().into_array());
    assert_eq!(root.subdivide_iter().collect::<Vec<_>>(), root.subdivide().into_iter().collect::<Vec<_>>());
    for sub_1 in root.subdivide_iter() {
      assert_eq!(sub_1.subdivide_array(), sub_1.subdivide().into_array());
      assert_eq!(sub_1.subdivide_iter().collect::<Vec<_>>(), sub_1.subdivide().into_iter().collect::<Vec<_>>());
      for sub_2 in sub_1.subdivide_iter() {
        assert_eq!(sub_2.subdivide_array(), sub_2.subdivide().into_array());
        assert_eq!(sub_2.subdivide_iter().collect::<Vec<_>>(), sub_2.subdivide().into_iter().collect::<Vec<_>>());
      }
    }
  }

  #[test]
  fn subdivide_iter() {
    let root = Aabb::root();
    assert_eq!(root.subdivide().into_iter().collect::<Vec<_>>(), root.subdivide_iter().collect::<Vec<_>>());
    assert_eq!(root.subdivide_array().into_iter().collect::<Vec<_>>(), root.subdivide_iter().collect::<Vec<_>>());
    for sub_1 in root.subdivide_iter() {
      assert_eq!(sub_1.subdivide().into_iter().collect::<Vec<_>>(), sub_1.subdivide_iter().collect::<Vec<_>>());
      assert_eq!(sub_1.subdivide_array().into_iter().collect::<Vec<_>>(), sub_1.subdivide_iter().collect::<Vec<_>>());
      for sub_2 in sub_1.subdivide_iter() {
        assert_eq!(sub_2.subdivide().into_iter().collect::<Vec<_>>(), sub_2.subdivide_iter().collect::<Vec<_>>());
        assert_eq!(sub_2.subdivide_array().into_iter().collect::<Vec<_>>(), sub_2.subdivide_iter().collect::<Vec<_>>());
      }
    }
  }


  #[test]
  fn twice_nested_siblings() {
    let sub_1 = Aabb::root().subdivide();
    let xyz_1 = sub_1.xyz;
    let yz_1 = sub_1.yz;
    let sub_2_in_xyz_1 = xyz_1.subdivide();
    let sub_2_in_yz_1 = yz_1.subdivide();
    assert_eq!(Some(sub_2_in_xyz_1.yz), sub_2_in_yz_1.xyz.sibling_positive_x());
  }

  #[test]
  fn trice_nested_siblings_1() {
    let root = Aabb::root();
    let sub_1 = root.subdivide();
    let xyz_depth_1 = sub_1.xyz;
    let sub_2_xyz_1 = xyz_depth_1.subdivide();
    let xyz_2_xyz_1 = sub_2_xyz_1.xyz;
    let yz_2_xyz_1 = sub_2_xyz_1.yz;
    let sub_3_xyz_2_xyz_1 = xyz_2_xyz_1.subdivide();
    let sub_3_yz_2_xyz_1 = yz_2_xyz_1.subdivide();
    let yz_3_xyz_2_xyz_1 = sub_3_xyz_2_xyz_1.yz;
    let xyz_3_yz_2_xyz_1 = sub_3_yz_2_xyz_1.xyz;
    assert_eq!(Some(yz_3_xyz_2_xyz_1), xyz_3_yz_2_xyz_1.sibling_positive_x());
  }

  #[test]
  fn trice_nested_siblings_2() {
    let root_size = 4096;
    let root = Aabb::root();
    let sub_1 = root.subdivide();
    let yz_1 = sub_1.yz;
    let sub_2_yz_1 = yz_1.subdivide();
    let xyz_2_yz_1 = sub_2_yz_1.xyz;
    let sub_3_xyz_2_yz_1 = xyz_2_yz_1.subdivide();
    let xyz_3_xyz_2_yz_1 = sub_3_xyz_2_yz_1.xyz;
    assert_eq!(UVec3::new(root_size / 2, root_size, root_size), xyz_3_xyz_2_yz_1.maximum_point(root_size));
    assert!(xyz_3_xyz_2_yz_1.sibling_positive_x().is_some());
    let yz_3_yz_2_xyz_1 = sub_1.xyz.subdivide().yz.subdivide().yz;
    assert_eq!(Some(yz_3_yz_2_xyz_1), xyz_3_xyz_2_yz_1.sibling_positive_x());
  }

  #[test]
  fn user_bit() {
    let mut root = Aabb::root();
    assert_eq!(false, root.is_user_bit_set());
    assert_eq!(true, root.with_user_bit_set().is_user_bit_set());
    root.set_user_bit();
    assert_eq!(true, root.is_user_bit_set());
    assert_eq!(false, root.with_user_bit_unset().is_user_bit_set());
    root.unset_user_bit();
    assert_eq!(false, root.is_user_bit_set());

    let root_size = 4096;
    let sub_1_false = root.subdivide();
    test_subdivided(root_size, 1, root_size / 2, UVec3::zero(), false, sub_1_false);
    for i in 0..8u8 {
      test_subdivided(root_size, 2, root_size / 4, sub_1_false[i].minimum_point(root_size), false, sub_1_false[i].subdivide());
    }

    root.set_user_bit();
    let sub_1_true = root.subdivide();
    test_subdivided(root_size, 1, root_size / 2, UVec3::zero(), true, sub_1_true);
    for i in 0..8u8 {
      test_subdivided(root_size, 2, root_size / 4, sub_1_true[i].minimum_point(root_size), true, sub_1_true[i].subdivide());
    }
  }

  fn test_points(aabb: Aabb, root_size: u32, ) {
    { // Closest point to minimum point is always the minimum point itself.
      let point = aabb.minimum_point(root_size).into();
      assert_eq!(point, aabb.closest_point(root_size, point));
      assert_eq!(0.0, aabb.distance_from(root_size, point));
    }
    { // Closest point to center point is always the center point itself.
      let point = aabb.center_point(root_size).into();
      assert_eq!(point, aabb.closest_point(root_size, point));
      assert_eq!(0.0, aabb.distance_from(root_size, point));
    }
    { // Closest point to maximum point is always the maximum point itself.
      let point = aabb.maximum_point(root_size).into();
      assert_eq!(point, aabb.closest_point(root_size, point));
      assert_eq!(0.0, aabb.distance_from(root_size, point));
    }
  }

  fn test_point(aabb: AabbWithSize, point: Vec3, expected_point: Vec3, expected_distance: f32) {
    assert_eq!(expected_point, aabb.closest_point(point));
    assert_eq!(expected_distance, aabb.distance_from(point));
  }

  fn p2(d1: f32, d2: f32) -> f32 {
    (d1.powf(2.0) + d2.powf(2.0)).sqrt()
  }

  fn p3(d1: f32, d2: f32, d3: f32) -> f32 {
    (d1.powf(2.0) + d2.powf(2.0) + d3.powf(2.0)).sqrt()
  }

  #[test]
  fn closest_point_and_distance() {
    let root_size = 4096;
    let root = Aabb::root().with_size(root_size);
    test_points(root.inner, root_size);
    for sub_1 in root.subdivide() {
      test_points(sub_1, root_size);
      for sub_2 in sub_1.subdivide_iter() {
        test_points(sub_2, root_size);
      }
    }

    let minimum_point = root.minimum_point().into();
    test_point(root, Vec3::new(-5000.0, 0.0, 0.0), minimum_point, 5000.0);
    test_point(root, Vec3::new(-5000.0, -6000.0, 0.0), minimum_point, p2(5000.0, 6000.0));
    test_point(root, Vec3::new(-5000.0, -6000.0, -7000.0), minimum_point, p3(5000.0, 6000.0, 7000.0));
    let size = root_size as f32;
    test_point(root, Vec3::new(5000.0, 0.0, 0.0), Vec3::new(size, 0.0, 0.0), 5000.0 - size);
    test_point(root, Vec3::new(5000.0, 6000.0, 0.0), Vec3::new(size, size, 0.0), p2(5000.0 - size, 6000.0 - size));
    test_point(root, Vec3::new(5000.0, 6000.0, 7000.0), Vec3::new(size, size, size), p3(5000.0 - size, 6000.0 - size, 7000.0 - size));
    let half_size = (root_size / 2) as f32;
    test_point(root, Vec3::new(half_size, half_size, -half_size), Vec3::new(half_size, half_size, 0.0), half_size);
    test_point(root, Vec3::new(half_size, half_size, size * 2.0), Vec3::new(half_size, half_size, size), size);
  }

  #[test]
  fn size() {
    assert_eq!(4, size_of::<Aabb>());
    assert_eq!(4, size_of::<Option<Aabb>>());
  }
}
