use std::marker::PhantomData;

use ultraviolet::UVec3;

use crate::chunk::array::Index;

// Shape trait

pub trait Shape<I> {
  const W: u32;
  const H: u32;
  const D: u32;

  fn index_into_xyz(index: I) -> (u32, u32, u32);
  #[inline]
  fn index_into_pos(index: I) -> UVec3 {
    let (x, y, z) = Self::index_into_xyz(index);
    UVec3::new(x, y, z)
  }

  fn index_from_xyz(x: u32, y: u32, z: u32) -> I;
  #[inline]
  fn index_from_pos(pos: UVec3) -> I {
    Self::index_from_xyz(pos.x, pos.y, pos.z)
  }

  fn for_all(run: impl FnMut(u32, u32, u32, I));
}


// Shape implementation

#[repr(transparent)]
pub struct ConstShape<I: Index, const W: u32, const H: u32, const D: u32> {
  _phantom: PhantomData<I>,
}

impl<I: Index, const W: u32, const H: u32, const D: u32> Shape<I> for ConstShape<I, W, H, D> {
  const W: u32 = W;
  const H: u32 = H;
  const D: u32 = D;

  #[inline]
  fn index_into_xyz(index: I) -> (u32, u32, u32) {
    let mut i = index.into_u32();
    let z = i / (W * H);
    i -= z * (W * H);
    let y = i / W;
    let x = i % W;
    (x, y, z)
  }

  #[inline]
  fn index_from_xyz(x: u32, y: u32, z: u32) -> I {
    I::from_u32(index_from_xyz::<I, Self>(x, y, z))
  }

  #[inline]
  fn for_all(mut run: impl FnMut(u32, u32, u32, I)) {
    let mut i = 0;
    for z in 0..D {
      for y in 0..H {
        for x in 0..W {
          run(x, y, z, I::from_u32(i));
          i += 1;
        }
      }
    }
  }
}

// Defined outside of trait/impl to make it available as const.
#[inline]
pub(crate) const fn index_from_xyz<I: Index, S: Shape<I>>(x: u32, y: u32, z: u32) -> u32 {
  x + (S::W * y) + (S::W * S::H * z)
}
