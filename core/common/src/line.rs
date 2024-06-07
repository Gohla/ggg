use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
#[cfg(feature = "egui")]
impl Into<egui::Vec2> for LineDelta {
  fn into(self) -> egui::Vec2 {
    // Note: loss of precision due to conversion from f64 into f32.
    egui::Vec2::new(self.x as f32, self.y as f32)
  }
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

impl Sub<LineDelta> for LineDelta {
  type Output = LineDelta;
  #[inline]
  fn sub(self, rhs: LineDelta) -> Self::Output {
    LineDelta::new(self.x - rhs.x, self.y - rhs.y)
  }
}
impl SubAssign<LineDelta> for LineDelta {
  #[inline]
  fn sub_assign(&mut self, rhs: LineDelta) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

impl Mul<f64> for LineDelta {
  type Output = LineDelta;
  #[inline]
  fn mul(self, rhs: f64) -> Self::Output {
    LineDelta::new(self.x * rhs, self.y * rhs)
  }
}
impl MulAssign<f64> for LineDelta {
  #[inline]
  fn mul_assign(&mut self, rhs: f64) {
    self.x *= rhs;
    self.y *= rhs;
  }
}

impl Div<f64> for LineDelta {
  type Output = LineDelta;
  #[inline]
  fn div(self, rhs: f64) -> Self::Output {
    LineDelta::new(self.x / rhs, self.y / rhs)
  }
}
impl DivAssign<f64> for LineDelta {
  #[inline]
  fn div_assign(&mut self, rhs: f64) {
    self.x /= rhs;
    self.y /= rhs;
  }
}
