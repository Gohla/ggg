use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use crate::sampler::{EventSampler, ValueSampler};

// Instant

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Debug)]
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
  pub fn to(&self, later: Instant) -> Duration {
    Duration::from_ns((later.0 - self.0) as i64)
  }
}

// Duration

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Debug)]
pub struct Duration(i64);

impl Duration {
  pub fn zero() -> Duration { Duration(0) }

  pub fn from_ns(ns: i64) -> Duration { Duration(ns) }
  pub fn from_us(us: i64) -> Duration { Duration::from_ns(us * 1_000) }
  pub fn from_ms(ms: i64) -> Duration { Duration::from_ns(ms * 1_000_000) }
  pub fn from_s(s: i64) -> Duration { Duration::from_ns(s * 1_000_000_000) }

  pub fn as_ns(&self) -> i64 { self.0 }
  pub fn as_us(&self) -> f64 { self.0 as f64 / 1_000.0 }
  pub fn as_ms(&self) -> f64 { self.0 as f64 / 1_000_000.0 }
  pub fn as_s(&self) -> f64 { self.0 as f64 / 1_000_000_000.0 }
}

impl Add for Duration {
  type Output = Duration;
  fn add(self, rhs: Duration) -> Self::Output { Duration(self.0 + rhs.0) }
}

impl AddAssign for Duration {
  fn add_assign(&mut self, rhs: Duration) { self.0 += rhs.0 }
}

impl Sub for Duration {
  type Output = Duration;
  fn sub(self, rhs: Duration) -> Self::Output { Duration(self.0 - rhs.0) }
}

impl SubAssign for Duration {
  fn sub_assign(&mut self, rhs: Duration) { self.0 -= rhs.0 }
}

impl Mul<i64> for Duration {
  type Output = Duration;
  fn mul(self, rhs: i64) -> Self::Output { Duration(self.0 * rhs) }
}

impl Div<Duration> for Duration {
  type Output = f64;
  fn div(self, rhs: Duration) -> Self::Output { self.0 as f64 / rhs.0 as f64 }
}

impl Div<u64> for Duration {
  type Output = Duration;
  fn div(self, rhs: u64) -> Self::Output { Duration(self.0 / rhs as i64) }
}

impl Div<usize> for Duration {
  type Output = Duration;
  fn div(self, rhs: usize) -> Self::Output { Duration(self.0 / rhs as i64) }
}

impl Div<i64> for Duration {
  type Output = Duration;
  fn div(self, rhs: i64) -> Self::Output { Duration(self.0 / rhs) }
}

impl Div<isize> for Duration {
  type Output = Duration;
  fn div(self, rhs: isize) -> Self::Output { Duration(self.0 / rhs as i64) }
}

// Timer

pub struct Timer {
  start: Instant,
  last: Instant,
}

#[derive(Copy, Clone, Debug)]
pub struct Time {
  pub elapsed: Duration,
  pub delta: Duration,
}

impl Timer {
  pub fn new() -> Timer {
    let now = Instant::now();
    return Timer { start: now, last: now };
  }

  pub fn update(&mut self) -> Time {
    let now = Instant::now();
    let elapsed = self.start.to(now);
    let delta = self.last.to(now);
    self.last = now;
    Time { elapsed, delta }
  }
}

// Frame timer

pub struct FrameTimer {
  timer: Timer,
  count: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct FrameTime {
  pub elapsed: Duration,
  pub delta: Duration,
  pub count: u64,
}

impl FrameTimer {
  pub fn new() -> FrameTimer { FrameTimer { timer: Timer::new(), count: 0 } }

  pub fn frame(&mut self) -> FrameTime {
    let Time { elapsed, delta: frame_time } = self.timer.update();
    let frame_time = FrameTime { elapsed, delta: frame_time, count: self.count };
    self.count += 1;
    frame_time
  }
}

// Tick timer

pub struct TickTimer {
  start: Instant,
  time_target: Duration,
  accumulated_lag: Duration,
  count: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct TickTime {
  pub time_target: Duration,
  pub accumulated_lag: Duration,
  pub delta: Duration,
  pub count: u64,
}

impl TickTimer {
  pub fn new(tick_time_target: Duration) -> TickTimer {
    TickTimer {
      count: 0,
      start: Instant::now(),
      time_target: tick_time_target,
      accumulated_lag: Duration::zero(),
    }
  }


  pub fn update_lag(&mut self, frame_time: Duration) -> Duration {
    self.accumulated_lag += frame_time;
    self.accumulated_lag
  }

  pub fn num_upcoming_ticks(&self) -> u64 {
    (self.accumulated_lag / self.time_target).floor() as u64
  }

  pub fn should_tick(&self) -> bool {
    self.accumulated_lag >= self.time_target
  }

  pub fn tick_start(&mut self) -> u64 {
    self.start = Instant::now();
    self.count
  }

  pub fn tick_end(&mut self) -> TickTime {
    self.accumulated_lag -= self.time_target;
    let tick_time = TickTime {
      time_target: self.time_target,
      accumulated_lag: self.accumulated_lag,
      delta: self.start.to(Instant::now()),
      count: self.count,
    };
    self.count += 1;
    tick_time
  }


  pub fn time_target(&self) -> Duration {
    self.time_target
  }

  pub fn accumulated_lag(&self) -> Duration {
    self.accumulated_lag
  }

  pub fn extrapolation(&self) -> f64 {
    let lag_ns = self.accumulated_lag.as_ns();
    let target_ns = self.time_target.as_ns();
    lag_ns as f64 / target_ns as f64
  }
}

// Timing statistic

#[derive(Default)]
pub struct TimingStats {
  // Time
  pub elapsed_time: Duration,
  // Frame
  pub frame_count: u64,
  pub frame_time: ValueSampler<Duration>,
  // Tick
  pub tick_count: u64,
  pub tick_time_target: Duration,
  pub tick_time: ValueSampler<Duration>,
  pub tick_rate: EventSampler,
  pub accumulated_lag: Duration,
  // Render
  pub render_extrapolation: f64,
}

impl TimingStats {
  pub fn new() -> TimingStats { TimingStats::default() }

  pub fn frame(&mut self, frame_time: FrameTime) {
    self.elapsed_time = frame_time.elapsed;
    self.frame_count = frame_time.count;
    self.frame_time.add(frame_time.delta);
  }

  pub fn tick(&mut self, tick_time: TickTime) {
    self.tick_count = tick_time.count;
    self.tick_time_target = tick_time.time_target;
    self.tick_time.add(tick_time.delta);
    self.tick_rate.add(Instant::now())
  }

  pub fn tick_lag(&mut self, accumulated_lag: Duration, gfx_extrapolation: f64) {
    self.accumulated_lag = accumulated_lag;
    self.render_extrapolation = gfx_extrapolation;
  }
}
