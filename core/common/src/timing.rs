use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

// Instant

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Instant(u64);

impl Instant {
  #[cfg(not(target_arch = "wasm32"))]
  pub fn now() -> Instant {
    use std::time::SystemTime;
    let instant = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
      .expect("System clock was before 1970")
      .as_nanos() as u64;
    Instant(instant)
  }

  #[cfg(target_arch = "wasm32")]
  pub fn now() -> Instant {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = window
      .performance()
      .expect("performance should be available");
    let instant_ms = performance.now();
    let instant_ns = instant_ms * 1000000.0;
    Instant(instant_ns as u64)
  }

  #[inline]
  pub fn to(&self, later: Instant) -> Offset {
    Offset::from_ns((later.0 - self.0) as i64)
  }
}

// Offset

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Debug)]
pub struct Offset(i64);

impl Offset {
  pub fn zero() -> Offset { Offset(0) }

  pub fn from_ns(ns: i64) -> Offset { Offset(ns) }
  pub fn from_us(us: i64) -> Offset { Offset::from_ns(us * 1_000) }
  pub fn from_ms(ms: i64) -> Offset { Offset::from_ns(ms * 1_000_000) }
  pub fn from_s(s: i64) -> Offset { Offset::from_ns(s * 1_000_000_000) }

  pub fn as_ns(&self) -> i64 { self.0 }
  pub fn as_us(&self) -> f64 { self.0 as f64 / 1_000.0 }
  pub fn as_ms(&self) -> f64 { self.0 as f64 / 1_000_000.0 }
  pub fn as_s(&self) -> f64 { self.0 as f64 / 1_000_000_000.0 }
}

impl Add for Offset {
  type Output = Offset;
  fn add(self, rhs: Offset) -> Self::Output { Offset(self.0 + rhs.0) }
}

impl AddAssign for Offset {
  fn add_assign(&mut self, rhs: Offset) { self.0 += rhs.0 }
}

impl Sub for Offset {
  type Output = Offset;
  fn sub(self, rhs: Offset) -> Self::Output { Offset(self.0 - rhs.0) }
}

impl SubAssign for Offset {
  fn sub_assign(&mut self, rhs: Offset) { self.0 -= rhs.0 }
}

impl Mul<i64> for Offset {
  type Output = Offset;
  fn mul(self, rhs: i64) -> Self::Output { Offset(self.0 * rhs) }
}

impl Div<Offset> for Offset {
  type Output = f64;
  fn div(self, rhs: Offset) -> Self::Output { self.0 as f64 / rhs.0 as f64 }
}

impl Div<u64> for Offset {
  type Output = Offset;
  fn div(self, rhs: u64) -> Self::Output { Offset(self.0 / rhs as i64) }
}

impl Div<usize> for Offset {
  type Output = Offset;
  fn div(self, rhs: usize) -> Self::Output { Offset(self.0 / rhs as i64) }
}

impl Div<i64> for Offset {
  type Output = Offset;
  fn div(self, rhs: i64) -> Self::Output { Offset(self.0 / rhs) }
}

impl Div<isize> for Offset {
  type Output = Offset;
  fn div(self, rhs: isize) -> Self::Output { Offset(self.0 / rhs as i64) }
}

// Timer

#[derive(Debug)]
pub struct Timer {
  instant: Instant,
}

impl Default for Timer {
  #[inline]
  fn default() -> Self {
    Self { instant: Instant::now() }
  }
}
impl Timer {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl Timer {
  pub fn time(&self) -> Offset {
    let now = Instant::now();
    self.instant.to(now)
  }
  pub fn reset(&mut self) {
    let now = Instant::now();
    self.instant = now;
  }
  pub fn time_then_reset(&mut self) -> Offset {
    let now = Instant::now();
    let time = self.instant.to(now);
    self.instant = now;
    time
  }
}
