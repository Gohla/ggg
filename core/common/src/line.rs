use std::ops::{Add, AddAssign};

/// Line delta: delta (difference) measured in lines.
#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub struct LineDelta {
  pub x: f64,
  pub y: f64,
}

impl LineDelta {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self {
    debug_assert!(x.is_finite(), "X {} is not finite", x);
    debug_assert!(!x.is_nan(), "X {} is NaN", x);
    debug_assert!(y.is_finite(), "Y {} is not finite", y);
    debug_assert!(!y.is_nan(), "Y {} is NaN", y);
    Self { x, y }
  }

  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0.0 && self.y == 0.0 }
}

impl From<(f64, f64)> for LineDelta {
  #[inline]
  fn from((x, y): (f64, f64)) -> Self { Self::new(x, y) }
}
impl From<(f32, f32)> for LineDelta {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self { Self::new(x as _, y as _) }
}
impl From<(i32, i32)> for LineDelta {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}
impl From<[f64; 2]> for LineDelta {
  #[inline]
  fn from([x, y]: [f64; 2]) -> Self { Self::new(x, y) }
}
impl From<[f32; 2]> for LineDelta {
  #[inline]
  fn from([x, y]: [f32; 2]) -> Self { Self::new(x as _, y as _) }
}
impl From<[i32; 2]> for LineDelta {
  #[inline]
  fn from([x, y]: [i32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<LineDelta> for (f64, f64) {
  #[inline]
  fn from(l: LineDelta) -> Self { (l.x, l.y) }
}
impl From<LineDelta> for [f64; 2] {
  #[inline]
  fn from(l: LineDelta) -> Self { [l.x, l.y] }
}

impl Add<LineDelta> for LineDelta {
  type Output = LineDelta;
  #[inline]
  fn add(self, rhs: LineDelta) -> Self::Output {
    LineDelta::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl AddAssign<LineDelta> for LineDelta {
  #[inline]
  fn add_assign(&mut self, rhs: LineDelta) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}
