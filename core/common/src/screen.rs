#![allow(dead_code)]

use std::ops::{Add, AddAssign, Div, Mul, Sub};

//
// Scale (DPI) factor.
//

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Scale(f64);

impl Scale {
  #[inline]
  pub fn new(scale: f64) -> Self {
    debug_assert!(scale.is_sign_positive(), "Scale {} is not positive", scale);
    debug_assert!(scale.is_normal(), "Scale {} is not normal", scale);
    Scale(scale)
  }


  #[inline]
  pub fn is_identity(&self) -> bool { self.0 == 1.0 }
}

impl Mul<Scale> for f64 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self * rhs.0 }
}

impl Div<Scale> for f64 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self / rhs.0 }
}

impl Mul<Scale> for u64 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}

impl Div<Scale> for u64 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}

impl Mul<Scale> for i64 {
  type Output = f64;

  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}

impl Div<Scale> for i64 {
  type Output = f64;

  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}

impl From<f64> for Scale {
  fn from(scale: f64) -> Self { Scale(scale) }
}

impl From<u64> for Scale {
  fn from(scale: u64) -> Self { Scale(scale as _) }
}

impl From<f32> for Scale {
  fn from(scale: f32) -> Self { Scale(scale as _) }
}

impl From<u32> for Scale {
  fn from(scale: u32) -> Self { Scale(scale as _) }
}

impl From<Scale> for f64 {
  #[inline]
  fn from(scale: Scale) -> Self { scale.0 }
}

impl Default for Scale {
  #[inline]
  fn default() -> Self { Scale(1.0) }
}


//
// Size
//

// Physical size: size in physical (real) pixels on the device.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhysicalSize {
  pub width: u64,
  pub height: u64,
}

impl PhysicalSize {
  #[inline]
  pub fn new(width: u64, height: u64) -> Self { Self { width, height } }

  /// Loss of precision in physical size: conversion from f64 into u32.
  #[inline]
  pub fn from_logical<L: Into<LogicalSize>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalSize {
    let scale = scale.into();
    LogicalSize::new(self.width / scale, self.height / scale)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.width == 0 && self.height == 0 }

  #[inline]
  pub fn ratio(&self) -> f64 { self.width as f64 / self.height as f64 }
}

impl From<(u64, u64)> for PhysicalSize {
  #[inline]
  fn from((width, height): (u64, u64)) -> Self { Self::new(width, height) }
}

impl From<(u32, u32)> for PhysicalSize {
  #[inline]
  fn from((width, height): (u32, u32)) -> Self { Self::new(width as _, height as _) }
}

impl From<[u64; 2]> for PhysicalSize {
  #[inline]
  fn from([width, height]: [u64; 2]) -> Self { Self::new(width, height) }
}

impl From<[u32; 2]> for PhysicalSize {
  #[inline]
  fn from([width, height]: [u32; 2]) -> Self { Self::new(width as _, height as _) }
}

impl From<PhysicalSize> for (u64, u64) {
  #[inline]
  fn from(p: PhysicalSize) -> Self { (p.width as _, p.height as _) }
}

impl From<PhysicalSize> for (f64, f64) {
  #[inline]
  fn from(p: PhysicalSize) -> Self { (p.width as _, p.height as _) }
}

impl From<PhysicalSize> for [u64; 2] {
  #[inline]
  fn from(p: PhysicalSize) -> Self { [p.width as _, p.height as _] }
}

impl From<PhysicalSize> for [f64; 2] {
  #[inline]
  fn from(p: PhysicalSize) -> Self { [p.width as _, p.height as _] }
}


// Logical size: size after scaling. That is, the physical size divided by the scale factor.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub struct LogicalSize {
  pub width: f64,
  pub height: f64,
}

impl LogicalSize {
  #[inline]
  pub fn new(width: f64, height: f64) -> Self {
    debug_assert!(width.is_sign_positive(), "Width {} is not positive", width);
    debug_assert!(width.is_finite(), "Width {} is not finite", width);
    debug_assert!(!width.is_nan(), "Width is NaN");
    debug_assert!(height.is_sign_positive(), "Height {} is not positive", height);
    debug_assert!(height.is_finite(), "Height {} is not finite", height);
    debug_assert!(!height.is_nan(), "Height is NaN");
    Self { width, height }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalSize>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalSize {
    let scale = scale.into();
    PhysicalSize::new((self.width * scale).round() as _, (self.height * scale).round() as _)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.width == 0.0 && self.height == 0.0 }
}

impl From<(f64, f64)> for LogicalSize {
  #[inline]
  fn from((width, height): (f64, f64)) -> Self { Self::new(width, height) }
}

impl From<(f32, f32)> for LogicalSize {
  #[inline]
  fn from((width, height): (f32, f32)) -> Self { Self::new(width as _, height as _) }
}

impl From<(u64, u64)> for LogicalSize {
  #[inline]
  fn from((width, height): (u64, u64)) -> Self { Self::new(width as _, height as _) }
}

impl From<(u32, u32)> for LogicalSize {
  #[inline]
  fn from((width, height): (u32, u32)) -> Self { Self::new(width as _, height as _) }
}

impl From<[f64; 2]> for LogicalSize {
  #[inline]
  fn from([width, height]: [f64; 2]) -> Self { Self::new(width, height) }
}

impl From<[f32; 2]> for LogicalSize {
  #[inline]
  fn from([width, height]: [f32; 2]) -> Self { Self::new(width as _, height as _) }
}

impl From<[u64; 2]> for LogicalSize {
  #[inline]
  fn from([width, height]: [u64; 2]) -> Self { Self::new(width as _, height as _) }
}

impl From<[u32; 2]> for LogicalSize {
  #[inline]
  fn from([width, height]: [u32; 2]) -> Self { Self::new(width as _, height as _) }
}

impl From<LogicalSize> for (f64, f64) {
  #[inline]
  fn from(l: LogicalSize) -> Self { (l.width, l.height) }
}

impl From<LogicalSize> for [f64; 2] {
  #[inline]
  fn from(l: LogicalSize) -> Self { [l.width, l.height] }
}


// Screen size: combination of physical size, scale, and logical size.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ScreenSize {
  pub physical: PhysicalSize,
  pub scale: Scale,
  pub logical: LogicalSize,
}

impl ScreenSize {
  #[inline]
  pub fn new(physical: PhysicalSize, scale: Scale, logical: LogicalSize) -> Self { Self { physical, scale, logical } }

  #[inline]
  pub fn from_logical_scale<L: Into<LogicalSize>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalSize>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }

  #[inline]
  pub fn from_unscaled(width: u64, height: u64) -> Self {
    let physical = PhysicalSize::new(width, height);
    let scale = Scale::default();
    let logical = physical.into_logical(scale);
    Self::new(physical, scale, logical)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.physical.is_zero() && self.logical.is_zero() }
}

impl From<ScreenSize> for LogicalSize {
  #[inline]
  fn from(s: ScreenSize) -> Self { s.logical }
}

impl From<ScreenSize> for PhysicalSize {
  #[inline]
  fn from(s: ScreenSize) -> Self { s.physical }
}

impl From<ScreenSize> for Scale {
  #[inline]
  fn from(s: ScreenSize) -> Self { s.scale }
}


//
// Position
//

// Position in physical screen space.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhysicalPosition {
  pub x: i64,
  pub y: i64,
}

impl PhysicalPosition {
  #[inline]
  pub fn new(x: i64, y: i64) -> Self { Self { x, y } }

  #[inline]
  pub fn from_logical<L: Into<LogicalPosition>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalPosition {
    let scale = scale.into();
    LogicalPosition::new(self.x / scale, self.y / scale)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0 && self.y == 0 }
}

impl Add<PhysicalPosition> for PhysicalPosition {
  type Output = PhysicalPosition;

  #[inline]
  fn add(self, rhs: PhysicalPosition) -> Self::Output {
    PhysicalPosition::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl AddAssign<PhysicalPosition> for PhysicalPosition {
  #[inline]
  fn add_assign(&mut self, rhs: PhysicalPosition) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl Sub<PhysicalPosition> for PhysicalPosition {
  type Output = PhysicalDelta;

  #[inline]
  fn sub(self, rhs: PhysicalPosition) -> Self::Output {
    PhysicalDelta::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl From<(i64, i64)> for PhysicalPosition {
  #[inline]
  fn from((x, y): (i64, i64)) -> Self { Self::new(x, y) }
}

impl From<(i32, i32)> for PhysicalPosition {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<[i64; 2]> for PhysicalPosition {
  #[inline]
  fn from([x, y]: [i64; 2]) -> Self { Self::new(x, y) }
}

impl From<[i32; 2]> for PhysicalPosition {
  #[inline]
  fn from([x, y]: [i32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<PhysicalPosition> for (i64, i64) {
  #[inline]
  fn from(p: PhysicalPosition) -> Self { (p.x, p.y) }
}

impl From<PhysicalPosition> for (f64, f64) {
  #[inline]
  fn from(p: PhysicalPosition) -> Self { (p.x as _, p.y as _) }
}

impl From<PhysicalPosition> for [i64; 2] {
  #[inline]
  fn from(p: PhysicalPosition) -> Self { [p.x, p.y] }
}

impl From<PhysicalPosition> for [f64; 2] {
  #[inline]
  fn from(p: PhysicalPosition) -> Self { [p.x as _, p.y as _] }
}

// Position in logical screen space.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub struct LogicalPosition {
  pub x: f64,
  pub y: f64,
}

impl LogicalPosition {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self {
    debug_assert!(x.is_finite(), "X {} is not finite", x);
    debug_assert!(!x.is_nan(), "X {} is NaN", x);
    debug_assert!(y.is_finite(), "Y {} is not finite", y);
    debug_assert!(!y.is_nan(), "Y {} is NaN", y);
    Self { x, y }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalPosition>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  /// Loss of precision in physical position: conversion from f64 into i32.
  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalPosition {
    let scale = scale.into();
    PhysicalPosition::new((self.x * scale).round() as _, (self.y * scale).round() as _)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0.0 && self.y == 0.0 }
}

impl Add<LogicalPosition> for LogicalPosition {
  type Output = LogicalPosition;

  #[inline]
  fn add(self, rhs: LogicalPosition) -> Self::Output {
    LogicalPosition::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl AddAssign<LogicalPosition> for LogicalPosition {
  #[inline]
  fn add_assign(&mut self, rhs: LogicalPosition) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl Sub<LogicalPosition> for LogicalPosition {
  type Output = LogicalDelta;

  #[inline]
  fn sub(self, rhs: LogicalPosition) -> Self::Output {
    LogicalDelta::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl From<(f64, f64)> for LogicalPosition {
  #[inline]
  fn from((x, y): (f64, f64)) -> Self { Self::new(x, y) }
}

impl From<(f32, f32)> for LogicalPosition {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for LogicalPosition {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<[f64; 2]> for LogicalPosition {
  #[inline]
  fn from([x, y]: [f64; 2]) -> Self { Self::new(x, y) }
}

impl From<[f32; 2]> for LogicalPosition {
  #[inline]
  fn from([x, y]: [f32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<[i32; 2]> for LogicalPosition {
  #[inline]
  fn from([x, y]: [i32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<LogicalPosition> for (f64, f64) {
  #[inline]
  fn from(l: LogicalPosition) -> Self { (l.x, l.y) }
}

impl From<LogicalPosition> for [f64; 2] {
  #[inline]
  fn from(l: LogicalPosition) -> Self { [l.x, l.y] }
}


// Screen position: combination of physical position, scale, and logical position.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ScreenPosition {
  pub physical: PhysicalPosition,
  pub logical: LogicalPosition,
}

impl ScreenPosition {
  #[inline]
  pub fn new(physical: PhysicalPosition, logical: LogicalPosition) -> Self { Self { physical, logical } }

  #[inline]
  pub fn from_logical_scale<L: Into<LogicalPosition>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalPosition>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, logical)
  }

  #[inline]
  pub fn from_unscaled(x: i64, y: i64) -> Self {
    let physical = PhysicalPosition::new(x, y);
    let logical = physical.into_logical(Scale::default());
    Self::new(physical, logical)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.physical.is_zero() && self.logical.is_zero() }
}

impl Add<ScreenPosition> for ScreenPosition {
  type Output = ScreenPosition;

  #[inline]
  fn add(self, rhs: ScreenPosition) -> Self::Output {
    ScreenPosition::new(self.physical + rhs.physical, self.logical + rhs.logical)
  }
}

impl AddAssign<ScreenPosition> for ScreenPosition {
  #[inline]
  fn add_assign(&mut self, rhs: ScreenPosition) {
    self.physical += rhs.physical;
    self.logical += rhs.logical;
  }
}

impl Sub<ScreenPosition> for ScreenPosition {
  type Output = ScreenDelta;

  #[inline]
  fn sub(self, rhs: ScreenPosition) -> Self::Output {
    ScreenDelta::new(self.physical - rhs.physical, self.logical - rhs.logical)
  }
}

impl From<ScreenPosition> for LogicalPosition {
  #[inline]
  fn from(screen_position: ScreenPosition) -> Self { screen_position.logical }
}

impl From<ScreenPosition> for PhysicalPosition {
  #[inline]
  fn from(screen_position: ScreenPosition) -> Self { screen_position.physical }
}


//
// Delta
//

// Delta in physical screen space.

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhysicalDelta {
  pub x: i64,
  pub y: i64,
}

impl PhysicalDelta {
  #[inline]
  pub fn new(x: i64, y: i64) -> Self { Self { x, y } }

  /// Loss of precision in physical delta: conversion from f64 into i32.
  #[inline]
  pub fn from_logical<L: Into<LogicalDelta>, S: Into<Scale>>(logical: L, scale: S) -> Self { logical.into().into_physical(scale) }

  #[inline]
  pub fn into_logical<S: Into<Scale>>(self, scale: S) -> LogicalDelta {
    let scale = scale.into();
    LogicalDelta::new(self.x / scale, self.y / scale)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0 && self.y == 0 }
}

impl Add<PhysicalDelta> for PhysicalDelta {
  type Output = PhysicalDelta;

  #[inline]
  fn add(self, rhs: PhysicalDelta) -> Self::Output {
    PhysicalDelta::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl AddAssign<PhysicalDelta> for PhysicalDelta {
  #[inline]
  fn add_assign(&mut self, rhs: PhysicalDelta) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl From<(i64, i64)> for PhysicalDelta {
  #[inline]
  fn from((x, y): (i64, i64)) -> Self { Self::new(x, y) }
}

impl From<(i32, i32)> for PhysicalDelta {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<[i64; 2]> for PhysicalDelta {
  #[inline]
  fn from([x, y]: [i64; 2]) -> Self { Self::new(x, y) }
}

impl From<[i32; 2]> for PhysicalDelta {
  #[inline]
  fn from([x, y]: [i32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<PhysicalDelta> for (i64, i64) {
  #[inline]
  fn from(p: PhysicalDelta) -> Self { (p.x, p.y) }
}

impl From<PhysicalDelta> for [i64; 2] {
  #[inline]
  fn from(p: PhysicalDelta) -> Self { [p.x, p.y] }
}


// Delta in logical screen space.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub struct LogicalDelta {
  pub x: f64,
  pub y: f64,
}

impl LogicalDelta {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self {
    debug_assert!(x.is_finite(), "X {} is not finite", x);
    debug_assert!(!x.is_nan(), "X {} is NaN", x);
    debug_assert!(y.is_finite(), "Y {} is not finite", y);
    debug_assert!(!y.is_nan(), "Y {} is NaN", y);
    Self { x, y }
  }

  #[inline]
  pub fn from_physical<P: Into<PhysicalDelta>, S: Into<Scale>>(physical: P, scale: S) -> Self { physical.into().into_logical(scale) }

  #[inline]
  pub fn into_physical<S: Into<Scale>>(self, scale: S) -> PhysicalDelta {
    let scale = scale.into();
    PhysicalDelta::new((self.x * scale).round() as _, (self.y * scale).round() as _)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0.0 && self.y == 0.0 }
}

impl Add<LogicalDelta> for LogicalDelta {
  type Output = LogicalDelta;

  #[inline]
  fn add(self, rhs: LogicalDelta) -> Self::Output {
    LogicalDelta::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl AddAssign<LogicalDelta> for LogicalDelta {
  #[inline]
  fn add_assign(&mut self, rhs: LogicalDelta) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl From<(f64, f64)> for LogicalDelta {
  #[inline]
  fn from((x, y): (f64, f64)) -> Self { Self::new(x, y) }
}

impl From<(f32, f32)> for LogicalDelta {
  #[inline]
  fn from((x, y): (f32, f32)) -> Self { Self::new(x as _, y as _) }
}

impl From<(i32, i32)> for LogicalDelta {
  #[inline]
  fn from((x, y): (i32, i32)) -> Self { Self::new(x as _, y as _) }
}

impl From<[f64; 2]> for LogicalDelta {
  #[inline]
  fn from([x, y]: [f64; 2]) -> Self { Self::new(x, y) }
}

impl From<[f32; 2]> for LogicalDelta {
  #[inline]
  fn from([x, y]: [f32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<[i32; 2]> for LogicalDelta {
  #[inline]
  fn from([x, y]: [i32; 2]) -> Self { Self::new(x as _, y as _) }
}

impl From<LogicalDelta> for (f64, f64) {
  #[inline]
  fn from(l: LogicalDelta) -> Self { (l.x, l.y) }
}

impl From<LogicalDelta> for [f64; 2] {
  #[inline]
  fn from(l: LogicalDelta) -> Self { [l.x, l.y] }
}


// Screen delta: combination of physical delta, scale, and logical delta.

#[derive(Default, Copy, Clone, PartialOrd, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ScreenDelta {
  pub physical: PhysicalDelta,
  pub logical: LogicalDelta,
}

impl ScreenDelta {
  #[inline]
  pub fn new(physical: PhysicalDelta, logical: LogicalDelta) -> Self { Self { physical, logical } }

  #[inline]
  pub fn from_logical_scale<L: Into<LogicalDelta>, S: Into<Scale>>(logical: L, scale: S) -> Self {
    let logical = logical.into();
    let scale = scale.into();
    let physical = logical.into_physical(scale);
    Self::new(physical, logical)
  }

  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalDelta>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical.into_logical(scale);
    Self::new(physical, logical)
  }

  #[inline]
  pub fn from_unscaled(x: i64, y: i64) -> Self {
    let physical = PhysicalDelta::new(x, y);
    let logical = physical.into_logical(Scale::default());
    Self::new(physical, logical)
  }


  #[inline]
  pub fn is_zero(&self) -> bool { self.physical.is_zero() && self.logical.is_zero() }
}

impl Add<ScreenDelta> for ScreenDelta {
  type Output = ScreenDelta;

  #[inline]
  fn add(self, rhs: ScreenDelta) -> Self::Output {
    ScreenDelta::new(self.physical + rhs.physical, self.logical + rhs.logical)
  }
}

impl AddAssign<ScreenDelta> for ScreenDelta {
  #[inline]
  fn add_assign(&mut self, rhs: ScreenDelta) {
    self.physical += rhs.physical;
    self.logical += rhs.logical;
  }
}

impl From<ScreenDelta> for LogicalDelta {
  #[inline]
  fn from(screen_position: ScreenDelta) -> Self { screen_position.logical }
}

impl From<ScreenDelta> for PhysicalDelta {
  #[inline]
  fn from(screen_position: ScreenDelta) -> Self { screen_position.physical }
}
