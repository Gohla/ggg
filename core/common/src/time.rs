use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

// Instant

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Instant(u128);

impl Instant {
  #[cfg(not(target_arch = "wasm32"))]
  pub fn now() -> Self {
    use std::sync::LazyLock;
    use std::time::Instant as StdInstant;
    static EPOCH: LazyLock<StdInstant> = LazyLock::new(|| StdInstant::now());
    let duration_since_epoch = StdInstant::now() - *EPOCH;
    let nanoseconds_since_epoch = duration_since_epoch.as_nanos();
    Self(nanoseconds_since_epoch)
  }

  #[cfg(target_arch = "wasm32")]
  pub fn now() -> Instant {
    // TODO: use web-time crate
    let window = web_sys::window().expect("should have a window in this context");
    let performance = window
      .performance()
      .expect("performance should be available");
    let instant_ms = performance.now();
    let instant_ns = instant_ms * 1000000.0;
    Instant(instant_ns as u64)
  }

  #[inline]
  pub fn elapsed(self) -> Offset {
    Self::now() - self
  }
}

impl Add<Offset> for Instant {
  type Output = Self;
  #[inline]
  fn add(self, rhs: Offset) -> Self::Output { Self(self.0.saturating_add_signed(rhs.0)) }
}
impl AddAssign<Offset> for Instant {
  #[inline]
  fn add_assign(&mut self, rhs: Offset) { self.0 = self.0.saturating_add_signed(rhs.0) }
}

impl Sub for Instant {
  type Output = Offset;
  #[inline]
  fn sub(self, rhs: Self) -> Self::Output {
    Offset((self.0 as i128).saturating_sub_unsigned(rhs.0))
  }
}


// Offset

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Offset(i128);

impl Offset {
  #[inline]
  pub fn zero() -> Self { Self::default() }

  #[inline]
  pub fn from_nanoseconds(nanoseconds: i128) -> Self { Self(nanoseconds) }
  #[inline]
  pub fn from_microseconds(microseconds: i128) -> Self { Self(microseconds * 1_000) }
  #[inline]
  pub fn from_milliseconds(milliseconds: i128) -> Self { Self(milliseconds * 1_000_000) }
  #[inline]
  pub fn from_seconds(seconds: i128) -> Self { Self(seconds * 1_000_000_000) }

  #[inline]
  pub fn into_nanoseconds(self) -> i128 { self.0 }
  #[inline]
  pub fn into_microseconds(self) -> f64 { self.0 as f64 / 1_000.0 }
  #[inline]
  pub fn into_milliseconds(self) -> f64 { self.0 as f64 / 1_000_000.0 }
  #[inline]
  pub fn into_seconds(self) -> f64 { self.0 as f64 / 1_000_000_000.0 }
}

impl Add for Offset {
  type Output = Self;
  #[inline]
  fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}
impl AddAssign for Offset {
  #[inline]
  fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0 }
}

impl Sub for Offset {
  type Output = Self;
  #[inline]
  fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}
impl SubAssign for Offset {
  #[inline]
  fn sub_assign(&mut self, rhs: Self) { self.0 -= rhs.0 }
}

impl Mul<i128> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: i128) -> Self::Output { Self(self.0 * rhs) }
}
impl Mul<isize> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: isize) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<i64> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: i64) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<i32> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: i32) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<i16> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: i16) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<i8> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: i8) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<usize> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: usize) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<u64> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: u64) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<u32> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: u32) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<u16> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: u16) -> Self::Output { Self(self.0 * rhs as i128) }
}
impl Mul<u8> for Offset {
  type Output = Self;
  #[inline]
  fn mul(self, rhs: u8) -> Self::Output { Self(self.0 * rhs as i128) }
}

impl Div<i128> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: i128) -> Self::Output { Self(self.0 / rhs) }
}
impl Div<isize> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: isize) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<i64> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: i64) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<i32> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: i32) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<i16> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: i16) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<i8> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: i8) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<usize> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: usize) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<u64> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: u64) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<u32> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: u32) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<u16> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: u16) -> Self::Output { Self(self.0 / rhs as i128) }
}
impl Div<u8> for Offset {
  type Output = Self;
  #[inline]
  fn div(self, rhs: u8) -> Self::Output { Self(self.0 / rhs as i128) }
}

impl Div for Offset {
  type Output = f64;
  #[inline]
  fn div(self, rhs: Self) -> Self::Output { self.0 as f64 / rhs.0 as f64 }
}


// Stopwatch

#[derive(Debug)]
pub struct Stopwatch {
  instant: Instant,
}

impl Default for Stopwatch {
  #[inline]
  fn default() -> Self {
    Self { instant: Instant::now() }
  }
}
impl Stopwatch {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl Stopwatch {
  #[inline]
  pub fn elapsed(&self) -> Offset {
    self.instant.elapsed()
  }

  #[inline]
  pub fn reset(&mut self) {
    self.instant = Instant::now();
  }

  #[inline]
  pub fn elapsed_then_reset(&mut self) -> Offset {
    let now = Instant::now();
    let offset = now - self.instant;
    self.instant = now;
    offset
  }
}
