use std::num::NonZeroU32;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::size::ChunkSize;

#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AABB(NonZeroU32);

impl AABB {
  #[inline]
  pub fn root() -> Self { Self(unsafe { NonZeroU32::new_unchecked(2) }) }

  #[inline]
  pub fn depth(&self) -> u8 { ((31 - self.0.leading_zeros()) / 3) as u8 }

  #[inline]
  pub fn half_size(&self, root_half_size: u32) -> u32 { Self::half_size_internal(root_half_size, self.depth()) }
  #[inline]
  pub fn size(&self, root_half_size: u32) -> u32 { Self::size_internal(root_half_size, self.depth()) }
  #[inline]
  pub fn minimum_point(&self, root_half_size: u32) -> UVec3 {
    let depth = self.depth();
    let size = Self::size_internal(root_half_size, depth);
    Self::minimum_point_internal(depth, size, self.0.get())
  }
  #[inline]
  pub fn center_point(&self, root_half_size: u32) -> UVec3 {
    let minimum_point = self.minimum_point(root_half_size);
    let half_size = self.half_size(root_half_size);
    Self::center_point_internal(minimum_point, half_size)
  }
  #[inline]
  pub fn maximum_point(&self, root_half_size: u32) -> UVec3 {
    let minimum_point = self.minimum_point(root_half_size);
    let size = self.size(root_half_size);
    Self::maximum_point_internal(minimum_point, size)
  }
  #[inline]
  pub fn step<C: ChunkSize>(&self, root_half_size: u32) -> u32 { self.size(root_half_size) / C::CELLS_IN_CHUNK_ROW }
  #[inline]
  pub fn closest_point(&self, root_half_size: u32, mut point: Vec3) -> Vec3 {
    let depth = self.depth();
    let half_size = Self::half_size_internal(root_half_size, depth);
    let minimum_point = Self::minimum_point_internal(depth, half_size, self.0.get());
    let maximum_point = Self::maximum_point_internal(minimum_point, half_size * 2);
    point.clamp(minimum_point.into(), maximum_point.into());
    point
  }
  #[inline]
  pub fn distance_from(&self, root_half_size: u32, point: Vec3) -> f32 {
    let depth = self.depth();
    let half_size = Self::half_size_internal(root_half_size, depth);
    let minimum_point = Self::minimum_point_internal(depth, half_size, self.0.get());
    let maximum_point = Self::maximum_point_internal(minimum_point, half_size * 2);
    let minimum_point: Vec3 = minimum_point.into();
    let maximum_point: Vec3 = maximum_point.into();
    let dx = (minimum_point.x - point.x).max(point.x - maximum_point.x).max(0.0);
    let dy = (minimum_point.y - point.y).max(point.y - maximum_point.y).max(0.0);
    let dz = (minimum_point.z - point.z).max(point.z - maximum_point.z).max(0.0);
    (dx * dx + dy * dy + dz * dz).sqrt()
  }
  #[inline]
  pub fn as_sized(&self, root_half_size: u32) -> AABBSized {
    AABBSized { root_half_size, inner: *self }
  }

  #[inline]
  pub fn subdivide(&self) -> AABBIter {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    AABBIter::new(self)
  }
  #[inline]
  pub fn subdivide_array(&self) -> [AABB; 8] {
    debug_assert!(self.0.leading_zeros() > 3, "Cannot subdivide {:?}, there is no space left in the locational code", self);
    let code = self.0.get();
    let user_bit_set = code & 1 != 0;
    let code = (code << 3) | user_bit_set as u32;
    unsafe {
      [
        Self::new_unchecked(code | 0b00000000),
        Self::new_unchecked(code | 0b00000010),
        Self::new_unchecked(code | 0b00000100),
        Self::new_unchecked(code | 0b00000110),
        Self::new_unchecked(code | 0b00001000),
        Self::new_unchecked(code | 0b00001010),
        Self::new_unchecked(code | 0b00001100),
        Self::new_unchecked(code | 0b00001110),
      ]
    }
  }

  #[inline]
  pub fn sibling_positive_x(&self) -> Option<Self> { self.positive_sibling::<1>() }
  #[inline]
  pub fn sibling_positive_y(&self) -> Option<Self> { self.positive_sibling::<2>() }
  #[inline]
  pub fn sibling_positive_z(&self) -> Option<Self> { self.positive_sibling::<3>() }
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
      let bit = 1 << ((d * 3) + O);
      if code & bit != 0 { // If bit is set, unset it to go to the positive sibling; and we're done.
        unsafe { return Some(Self::new_unchecked(code & !bit)); }  // TODO: preserve user bit and test this
      } else { // Otherwise set the bit to go to the negative sibling and continue.
        code = code | bit;
      }
    }
    None // No parent was found with the bit set, so we couldn't go to a positive sibling anywhere.
  }

  #[inline]
  unsafe fn new_unchecked(code: u32) -> Self {
    debug_assert!(code != 0);
    Self(NonZeroU32::new_unchecked(code))
  }
  #[inline]
  fn size_internal(root_half_size: u32, depth: u8) -> u32 {
    debug_assert!(root_half_size.is_power_of_two(), "Root half size {} is not a power of 2", root_half_size);
    (root_half_size << 1) >> depth // Left shift is multiply by 2, right shift is divide by 2; for powers of 2.
  }
  #[inline]
  fn half_size_internal(root_half_size: u32, depth: u8) -> u32 {
    debug_assert!(root_half_size.is_power_of_two(), "Root half size {} is not a power of 2", root_half_size);
    root_half_size >> depth // Right shift is divide by 2 for powers of 2.
  }
  #[inline]
  fn minimum_point_internal(depth: u8, mut size: u32, mut code: u32) -> UVec3 {
    let mut minimum_point = UVec3::zero();
    for _ in 0..depth {
      let octant = code as u8 & 0b00001110;
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
      0b00000000 => UVec3::new(size, size, size),
      0b00000010 => UVec3::new(0, size, size),
      0b00000100 => UVec3::new(size, 0, size),
      0b00000110 => UVec3::new(0, 0, size),
      0b00001000 => UVec3::new(size, size, 0),
      0b00001010 => UVec3::new(0, size, 0),
      0b00001100 => UVec3::new(size, 0, 0),
      0b00001110 => UVec3::new(0, 0, 0),
      _ => unreachable!(),
    }
  }
}


// Sized AABB

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AABBSized {
  pub root_half_size: u32,
  pub inner: AABB,
}

impl AABBSized {
  #[inline]
  pub fn depth(&self) -> u8 { self.inner.depth() }

  #[inline]
  pub fn half_size(&self) -> u32 { self.inner.half_size(self.root_half_size) }
  #[inline]
  pub fn size(&self) -> u32 { self.inner.size(self.root_half_size) }
  #[inline]
  pub fn minimum_point(&self) -> UVec3 { self.inner.minimum_point(self.root_half_size) }
  #[inline]
  pub fn center_point(&self) -> UVec3 { self.inner.center_point(self.root_half_size) }
  #[inline]
  pub fn maximum_point(&self) -> UVec3 { self.inner.maximum_point(self.root_half_size) }
  #[inline]
  pub fn step<C: ChunkSize>(&self) -> u32 { self.inner.step::<C>(self.root_half_size) }
  #[inline]
  pub fn closest_point(&self, point: Vec3) -> Vec3 { self.inner.closest_point(self.root_half_size, point) }
  #[inline]
  pub fn distance_from(&self, point: Vec3) -> f32 { self.inner.distance_from(self.root_half_size, point) }

  #[inline]
  pub fn subdivide(&self) -> AABBIter { self.inner.subdivide() }
  #[inline]
  pub fn subdivide_array(&self) -> [AABB; 8] { self.inner.subdivide_array() }
  #[inline]
  pub fn sibling_positive_x(&self) -> Option<Self> { self.inner.sibling_positive_x().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
  #[inline]
  pub fn sibling_positive_y(&self) -> Option<Self> { self.inner.sibling_positive_y().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
  #[inline]
  pub fn sibling_positive_z(&self) -> Option<Self> { self.inner.sibling_positive_z().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
  #[inline]
  pub fn sibling_positive_xy(&self) -> Option<Self> { self.inner.sibling_positive_xy().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
  #[inline]
  pub fn sibling_positive_yz(&self) -> Option<Self> { self.inner.sibling_positive_yz().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
  #[inline]
  pub fn sibling_positive_xz(&self) -> Option<Self> { self.inner.sibling_positive_xz().map(|inner| Self { root_half_size: self.root_half_size, inner }) }
}


// Iterator

pub struct AABBIter {
  code: u32,
  octant: u8,
}

impl AABBIter {
  #[inline]
  fn new(aabb: &AABB) -> Self { Self { code: aabb.0.get() << 3, octant: 1 } }
}

impl Iterator for AABBIter {
  type Item = AABB;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> { // TODO: walk over the correct octants and test this. (i.e., test that both subdivide methods return the same things)
    if self.octant > 8 { return None; } 
    let aabb = unsafe { AABB::new_unchecked(self.code | self.octant as u32) };  // TODO: preserve user bit and test this
    self.octant += 1;
    Some(aabb)
  }
}


// Tests

#[cfg(test)]
mod tests {
  use std::mem::size_of;

  use ultraviolet::UVec3;

  use crate::lod::aabb::AABB;

  #[test]
  fn root() {
    let root_half_size = 2048;
    let size = root_half_size * 2;
    let root = AABB::root().as_sized(root_half_size);
    assert_eq!(0, root.depth());
    assert_eq!(root_half_size, root.half_size());
    assert_eq!(size, root.size());
    assert_eq!(UVec3::zero(), root.minimum_point());
    assert_eq!(UVec3::broadcast(root_half_size), root.center_point());
    assert_eq!(UVec3::broadcast(size), root.maximum_point());
    assert_eq!(None, root.sibling_positive_x());
    assert_eq!(None, root.sibling_positive_y());
    assert_eq!(None, root.sibling_positive_z());
    assert_eq!(None, root.sibling_positive_xy());
    assert_eq!(None, root.sibling_positive_yz());
    assert_eq!(None, root.sibling_positive_xz());
  }

  fn test_subdivided(root_half_size: u32, depth: u8, half_size: u32, offset: UVec3, subdivided: [AABB; 8]) {
    let size = half_size * 2;

    let front = {
      let sub = subdivided[0].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, size, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, size + half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size * 2, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.sibling_positive_x());
        assert_eq!(None, sub.sibling_positive_y());
        assert_eq!(None, sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };
    let front_x = {
      let sub = subdivided[1].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, size, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, size + half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size * 2, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(front), sub.sibling_positive_x());
        assert_eq!(None, sub.sibling_positive_y());
        assert_eq!(None, sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };
    let front_y = {
      let sub = subdivided[2].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, 0, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.sibling_positive_x());
        assert_eq!(Some(front), sub.sibling_positive_y());
        assert_eq!(None, sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };
    let front_xy = {
      let sub = subdivided[3].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, 0, size), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, half_size, size + half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size, size * 2), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(front_y), sub.sibling_positive_x());
        assert_eq!(Some(front_x), sub.sibling_positive_y());
        assert_eq!(None, sub.sibling_positive_z());
        assert_eq!(Some(front), sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };

    let back = {
      let sub = subdivided[4].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, size, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, size + half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size * 2, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.sibling_positive_x());
        assert_eq!(None, sub.sibling_positive_y());
        assert_eq!(Some(front), sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };
    let back_x = {
      let sub = subdivided[5].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, size, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, size + half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size * 2, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(back), sub.sibling_positive_x());
        assert_eq!(None, sub.sibling_positive_y());
        assert_eq!(Some(front_x), sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(None, sub.sibling_positive_yz());
        assert_eq!(Some(front), sub.sibling_positive_xz());
      }
      sub
    };
    let back_y = {
      let sub = subdivided[6].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(size, 0, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(size + half_size, half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size * 2, size, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(None, sub.sibling_positive_x());
        assert_eq!(Some(back), sub.sibling_positive_y());
        assert_eq!(Some(front_y), sub.sibling_positive_z());
        assert_eq!(None, sub.sibling_positive_xy());
        assert_eq!(Some(front), sub.sibling_positive_yz());
        assert_eq!(None, sub.sibling_positive_xz());
      }
      sub
    };
    let _back_xy = {
      let sub = subdivided[7].as_sized(root_half_size);
      assert_eq!(depth, sub.depth());
      assert_eq!(half_size, sub.half_size());
      assert_eq!(size, sub.size());
      assert_eq!(offset + UVec3::new(0, 0, 0), sub.minimum_point());
      assert_eq!(offset + UVec3::new(half_size, half_size, half_size), sub.center_point());
      assert_eq!(offset + UVec3::new(size, size, size), sub.maximum_point());
      if depth == 1 {
        assert_eq!(Some(back_y), sub.sibling_positive_x());
        assert_eq!(Some(back_x), sub.sibling_positive_y());
        assert_eq!(Some(front_xy), sub.sibling_positive_z());
        assert_eq!(Some(back), sub.sibling_positive_xy());
        assert_eq!(Some(front_x), sub.sibling_positive_yz());
        assert_eq!(Some(front_y), sub.sibling_positive_xz());
      }
      sub
    };
  }

  #[test]
  fn subdivide_once() {
    let root_half_size = 2048;
    test_subdivided(root_half_size, 1, 1024, UVec3::zero(), AABB::root().subdivide_array());
  }

  #[test]
  fn subdivide_twice() {
    let root_half_size = 2048;
    for sub in AABB::root().subdivide_array() {
      test_subdivided(root_half_size, 2, 512, sub.minimum_point(root_half_size), sub.subdivide_array());
    }
  }

  #[test]
  fn subdivide_trice() {
    let root_half_size = 2048;
    for sub_1 in AABB::root().subdivide_array() {
      for sub_2 in sub_1.subdivide_array() {
        test_subdivided(root_half_size, 3, 256, sub_2.minimum_point(root_half_size), sub_2.subdivide_array());
      }
    }
  }

  #[test]
  fn twice_nested_siblings() {
    let subdivided_1 = AABB::root().subdivide_array();
    let front_1 = subdivided_1[0];
    let front_x_1 = subdivided_1[1];
    let subdivided_2_in_front_1 = front_1.subdivide_array();
    let subdivided_2_in_front_x_1 = front_x_1.subdivide_array();
    assert_eq!(Some(subdivided_2_in_front_1[1]), subdivided_2_in_front_x_1[0].sibling_positive_x());
  }

  #[test]
  fn trice_nested_siblings_1() {
    let root = AABB::root();
    let sub_1 = root.subdivide_array();
    let front_depth_1 = sub_1[0];
    let sub_2_front_1 = front_depth_1.subdivide_array();
    let front_2_front_1 = sub_2_front_1[0];
    let front_x_2_front_1 = sub_2_front_1[1];
    let sub_3_front_2_front_1 = front_2_front_1.subdivide_array();
    let sub_3_front_x_2_front_1 = front_x_2_front_1.subdivide_array();
    let front_x_3_front_2_front_1 = sub_3_front_2_front_1[1];
    let front_3_front_x_2_front_1 = sub_3_front_x_2_front_1[0];
    assert_eq!(Some(front_x_3_front_2_front_1), front_3_front_x_2_front_1.sibling_positive_x());
  }

  #[test]
  fn trice_nested_siblings_2() {
    let root_half_size = 2048;
    let root = AABB::root();
    let sub_1 = root.subdivide_array();
    let front_x_1 = sub_1[1];
    let sub_2_front_x_1 = front_x_1.subdivide_array();
    let front_2_front_x_1 = sub_2_front_x_1[0];
    let sub_3_front_2_front_x_1 = front_2_front_x_1.subdivide_array();
    let front_3_front_2_front_x_1 = sub_3_front_2_front_x_1[0];
    assert_eq!(UVec3::new(root_half_size, root_half_size * 2, root_half_size * 2), front_3_front_2_front_x_1.maximum_point(root_half_size));
    assert!(front_3_front_2_front_x_1.sibling_positive_x().is_some());
    let front_3_x_front_2_x_front_1 = sub_1[0].subdivide_array()[1].subdivide_array()[1];
    assert_eq!(Some(front_3_x_front_2_x_front_1), front_3_front_2_front_x_1.sibling_positive_x());
  }

  #[test]
  fn size() {
    assert_eq!(4, size_of::<AABB>());
    assert_eq!(4, size_of::<Option<AABB>>());
  }
}
