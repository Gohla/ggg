#![allow(dead_code)]

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Scale factor. The number of physical pixels per logical point.
///
/// Sometimes called DPI (Dots Per Inch), but this is a misnomer because this scale has nothing to do with the surface
/// area of the screen.
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

impl Default for Scale {
  #[inline]
  fn default() -> Self { Scale(1.0) }
}

impl From<f64> for Scale {
  #[inline]
  fn from(scale: f64) -> Self { Scale(scale) }
}
impl From<u64> for Scale {
  #[inline]
  fn from(scale: u64) -> Self { Scale(scale as _) }
}
impl From<f32> for Scale {
  #[inline]
  fn from(scale: f32) -> Self { Scale(scale as _) }
}
impl From<u32> for Scale {
  #[inline]
  fn from(scale: u32) -> Self { Scale(scale as _) }
}
impl From<u16> for Scale {
  #[inline]
  fn from(scale: u16) -> Self { Scale(scale as _) }
}
impl From<u8> for Scale {
  #[inline]
  fn from(scale: u8) -> Self { Scale(scale as _) }
}

impl From<Scale> for f64 {
  #[inline]
  fn from(scale: Scale) -> Self { scale.0 }
}

impl Mul<Scale> for f64 {
  type Output = f64;
  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self * rhs.0 }
}
impl Mul<Scale> for u64 {
  type Output = f64;
  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}
impl Mul<Scale> for i64 {
  type Output = f64;
  #[inline]
  fn mul(self, rhs: Scale) -> f64 { self as f64 * rhs.0 }
}

impl Div<Scale> for f64 {
  type Output = f64;
  #[inline]
  fn div(self, rhs: Scale) -> f64 { self / rhs.0 }
}
impl Div<Scale> for u64 {
  type Output = f64;
  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}
impl Div<Scale> for i64 {
  type Output = f64;
  #[inline]
  fn div(self, rhs: Scale) -> f64 { self as f64 / rhs.0 }
}


//
// Size
//

/// Physical size: size measured in physical (actual/real) pixels on the device.
///
/// Divide [physical size](PhysicalSize) by the [scale factor](Scale) to convert it a [logical size](LogicalSize).
/// Conversely, multiply [logical size](LogicalSize) by the [scale factor](Scale) to convert it a
/// [physical size](PhysicalSize).
///
/// For example, the actual resolution of your display/monitor is a physical size.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhysicalSize {
  pub width: u64,
  pub height: u64,
}

impl PhysicalSize {
  #[inline]
  pub fn new(width: u64, height: u64) -> Self { Self { width, height } }

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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> From<winit::dpi::PhysicalSize<P>> for PhysicalSize {
  #[inline]
  fn from(size: winit::dpi::PhysicalSize<P>) -> Self {
    let size: (u32, u32) = size.into();
    Self::from(size)
  }
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> Into<winit::dpi::PhysicalSize<P>> for PhysicalSize {
  #[inline]
  fn into(self) -> winit::dpi::PhysicalSize<P> {
    // Note: loss of precision due to conversion from u64 into u32.
    winit::dpi::PhysicalSize::from((self.width as u32, self.height as u32))
  }
}
#[cfg(feature = "winit")]
impl Into<winit::dpi::Size> for PhysicalSize {
  #[inline]
  fn into(self) -> winit::dpi::Size {
    // Note: loss of precision due to conversion from u64 into u32.
    winit::dpi::Size::Physical(self.into())
  }
}
#[cfg(feature = "egui")]
impl Into<egui::Vec2> for PhysicalSize {
  fn into(self) -> egui::Vec2 {
    // Note: loss of precision due to conversion from u64 into f32.
    egui::Vec2::new(self.width as f32, self.height as f32)
  }
}

impl Div<Scale> for PhysicalSize {
  type Output = LogicalSize;
  #[inline]
  fn div(self, rhs: Scale) -> Self::Output {
    LogicalSize::new(self.width / rhs, self.height / rhs)
  }
}

impl Mul<u64> for PhysicalSize {
  type Output = PhysicalSize;
  #[inline]
  fn mul(self, rhs: u64) -> Self::Output {
    Self::new(self.width * rhs, self.height * rhs)
  }
}


/// Logical size: size measured in logical (software) pixels.
///
/// Multiply [logical size](LogicalSize) by the [scale factor](Scale) to convert it a [physical size](PhysicalSize).
/// Conversely, divide [physical size](PhysicalSize) by the [scale factor](Scale) to convert it a
/// [logical size](LogicalSize).
///
/// For example, the resolution of a framebuffer is a logical size, which may differ from the resolution of your
/// display, and thus require scaling to fit to your display.
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> From<winit::dpi::LogicalSize<P>> for LogicalSize {
  #[inline]
  fn from(size: winit::dpi::LogicalSize<P>) -> Self {
    let size: (f64, f64) = size.into();
    Self::from(size)
  }
}

impl From<LogicalSize> for (f64, f64) {
  #[inline]
  fn from(l: LogicalSize) -> Self { (l.width, l.height) }
}
impl From<LogicalSize> for [f64; 2] {
  #[inline]
  fn from(l: LogicalSize) -> Self { [l.width, l.height] }
}
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> Into<winit::dpi::LogicalSize<P>> for LogicalSize {
  #[inline]
  fn into(self) -> winit::dpi::LogicalSize<P> {
    let size: (f64, f64) = self.into();
    winit::dpi::LogicalSize::from(size)
  }
}
#[cfg(feature = "winit")]
impl Into<winit::dpi::Size> for LogicalSize {
  #[inline]
  fn into(self) -> winit::dpi::Size {
    winit::dpi::Size::Logical(self.into())
  }
}
#[cfg(feature = "egui")]
impl Into<egui::Vec2> for LogicalSize {
  fn into(self) -> egui::Vec2 {
    // Note: loss of precision due to conversion from u64 into f32.
    egui::Vec2::new(self.width as f32, self.height as f32)
  }
}

impl Mul<Scale> for LogicalSize {
  type Output = PhysicalSize;
  #[inline]
  fn mul(self, rhs: Scale) -> Self::Output {
    // Note: rounds f64 into u64.
    PhysicalSize::new((self.width * rhs).round() as _, (self.height * rhs).round() as _)
  }
}

impl Mul<f64> for LogicalSize {
  type Output = LogicalSize;
  #[inline]
  fn mul(self, rhs: f64) -> Self::Output {
    Self::new(self.width * rhs, self.height * rhs)
  }
}
impl MulAssign<f64> for LogicalSize {
  #[inline]
  fn mul_assign(&mut self, rhs: f64) {
    self.width *= rhs;
    self.height *= rhs;
  }
}
impl Div<f64> for LogicalSize {
  type Output = LogicalSize;
  #[inline]
  fn div(self, rhs: f64) -> Self::Output {
    Self::new(self.width / rhs, self.height / rhs)
  }
}
impl DivAssign<f64> for LogicalSize {
  #[inline]
  fn div_assign(&mut self, rhs: f64) {
    self.width /= rhs;
    self.height /= rhs;
  }
}


/// Screen size: combination of physical size, scale, and logical size.
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
    let physical = logical * scale;
    Self::new(physical, scale, logical)
  }
  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalSize>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let scale = scale.into();
    let logical = physical / scale;
    Self::new(physical, scale, logical)
  }
  #[inline]
  pub fn from_unscaled(width: u64, height: u64) -> Self {
    let physical = PhysicalSize::new(width, height);
    let scale = Scale::default();
    let logical = physical / scale;
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

/// Physical position: position measured in physical (actual/real) pixels on the device.
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
  pub fn is_zero(&self) -> bool { self.x == 0 && self.y == 0 }
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> From<winit::dpi::PhysicalPosition<P>> for PhysicalPosition {
  #[inline]
  fn from(position: winit::dpi::PhysicalPosition<P>) -> Self {
    let position: (i32, i32) = position.into();
    Self::from(position)
  }
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> Into<winit::dpi::PhysicalPosition<P>> for PhysicalPosition {
  #[inline]
  fn into(self) -> winit::dpi::PhysicalPosition<P> {
    // Note: loss of precision due to conversion from i64 to i32.
    winit::dpi::PhysicalPosition::from((self.x as i32, self.y as i32))
  }
}
#[cfg(feature = "egui")]
impl Into<egui::Pos2> for PhysicalPosition {
  fn into(self) -> egui::Pos2 {
    // Note: loss of precision due to conversion from i64 into f32.
    egui::Pos2::new(self.x as f32, self.y as f32)
  }
}

impl Div<Scale> for PhysicalPosition {
  type Output = LogicalPosition;
  #[inline]
  fn div(self, rhs: Scale) -> Self::Output {
    LogicalPosition::new(self.x / rhs, self.y / rhs)
  }
}

impl Add<PhysicalDelta> for PhysicalPosition {
  type Output = PhysicalPosition;
  #[inline]
  fn add(self, rhs: PhysicalDelta) -> Self::Output {
    PhysicalPosition::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl AddAssign<PhysicalDelta> for PhysicalPosition {
  #[inline]
  fn add_assign(&mut self, rhs: PhysicalDelta) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}
impl Sub<PhysicalDelta> for PhysicalPosition {
  type Output = PhysicalPosition;
  #[inline]
  fn sub(self, rhs: PhysicalDelta) -> Self::Output {
    PhysicalPosition::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl SubAssign<PhysicalDelta> for PhysicalPosition {
  #[inline]
  fn sub_assign(&mut self, rhs: PhysicalDelta) {
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


/// Logical position: position measured in logical (software) pixels.
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
  pub fn is_zero(&self) -> bool { self.x == 0.0 && self.y == 0.0 }
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> From<winit::dpi::LogicalPosition<P>> for LogicalPosition {
  #[inline]
  fn from(position: winit::dpi::LogicalPosition<P>) -> Self {
    let position: (f64, f64) = position.into();
    Self::from(position)
  }
}

impl From<LogicalPosition> for (f64, f64) {
  #[inline]
  fn from(l: LogicalPosition) -> Self { (l.x, l.y) }
}
impl From<LogicalPosition> for [f64; 2] {
  #[inline]
  fn from(l: LogicalPosition) -> Self { [l.x, l.y] }
}
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> Into<winit::dpi::LogicalPosition<P>> for LogicalPosition {
  #[inline]
  fn into(self) -> winit::dpi::LogicalPosition<P> {
    let position: (f64, f64) = self.into();
    winit::dpi::LogicalPosition::from(position)
  }
}
#[cfg(feature = "egui")]
impl Into<egui::Pos2> for LogicalPosition {
  fn into(self) -> egui::Pos2 {
    // Note: loss of precision due to conversion from f64 into f32.
    egui::Pos2::new(self.x as f32, self.y as f32)
  }
}

impl Mul<Scale> for LogicalPosition {
  type Output = PhysicalPosition;
  #[inline]
  fn mul(self, rhs: Scale) -> Self::Output {
    // Note: rounds f64 into i64.
    PhysicalPosition::new((self.x * rhs).round() as _, (self.y * rhs).round() as _)
  }
}

impl Add<LogicalDelta> for LogicalPosition {
  type Output = LogicalPosition;
  #[inline]
  fn add(self, rhs: LogicalDelta) -> Self::Output {
    LogicalPosition::new(self.x + rhs.x, self.y + rhs.y)
  }
}
impl AddAssign<LogicalDelta> for LogicalPosition {
  #[inline]
  fn add_assign(&mut self, rhs: LogicalDelta) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}
impl Sub<LogicalDelta> for LogicalPosition {
  type Output = LogicalDelta;
  #[inline]
  fn sub(self, rhs: LogicalDelta) -> Self::Output {
    LogicalDelta::new(self.x - rhs.x, self.y - rhs.y)
  }
}
impl SubAssign<LogicalDelta> for LogicalPosition {
  #[inline]
  fn sub_assign(&mut self, rhs: LogicalDelta) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

impl Sub<LogicalPosition> for LogicalPosition {
  type Output = LogicalDelta;
  #[inline]
  fn sub(self, rhs: LogicalPosition) -> Self::Output {
    LogicalDelta::new(self.x - rhs.x, self.y - rhs.y)
  }
}


/// Screen position: combination of physical and logical position.
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
    let physical = logical * scale.into();
    Self::new(physical, logical)
  }
  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalPosition>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let logical = physical / scale.into();
    Self::new(physical, logical)
  }
  #[inline]
  pub fn from_unscaled(x: i64, y: i64) -> Self {
    let physical = PhysicalPosition::new(x, y);
    let logical = physical / Scale::default();
    Self::new(physical, logical)
  }

  #[inline]
  pub fn is_zero(&self) -> bool { self.physical.is_zero() && self.logical.is_zero() }
}

impl Add<ScreenDelta> for ScreenPosition {
  type Output = ScreenPosition;
  #[inline]
  fn add(self, rhs: ScreenDelta) -> Self::Output {
    ScreenPosition::new(self.physical + rhs.physical, self.logical + rhs.logical)
  }
}
impl AddAssign<ScreenDelta> for ScreenPosition {
  #[inline]
  fn add_assign(&mut self, rhs: ScreenDelta) {
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

/// Physical delta: delta (difference) measured in physical (actual/real) pixels on the device.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhysicalDelta {
  pub x: i64,
  pub y: i64,
}

impl PhysicalDelta {
  #[inline]
  pub fn new(x: i64, y: i64) -> Self { Self { x, y } }

  #[inline]
  pub fn is_zero(&self) -> bool { self.x == 0 && self.y == 0 }
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
#[cfg(feature = "winit")]
impl<P: winit::dpi::Pixel> From<winit::dpi::PhysicalPosition<P>> for PhysicalDelta {
  #[inline]
  fn from(position_as_delta: winit::dpi::PhysicalPosition<P>) -> Self {
    let (x, y): (f64, f64) = position_as_delta.into();
    // Note: rounds f64 to i64.
    Self::new(x.round() as i64, y.round() as i64)
  }
}

impl From<PhysicalDelta> for (i64, i64) {
  #[inline]
  fn from(p: PhysicalDelta) -> Self { (p.x, p.y) }
}
impl From<PhysicalDelta> for [i64; 2] {
  #[inline]
  fn from(p: PhysicalDelta) -> Self { [p.x, p.y] }
}
#[cfg(feature = "egui")]
impl Into<egui::Vec2> for PhysicalDelta {
  fn into(self) -> egui::Vec2 {
    // Note: loss of precision due to conversion from i64 into f32.
    egui::Vec2::new(self.x as f32, self.y as f32)
  }
}

impl Div<Scale> for PhysicalDelta {
  type Output = LogicalDelta;
  #[inline]
  fn div(self, rhs: Scale) -> Self::Output {
    LogicalDelta::new(self.x / rhs, self.y / rhs)
  }
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


/// Logical delta: delta (difference) measured in logical (software) pixels.
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
  pub fn is_zero(&self) -> bool { self.x == 0.0 && self.y == 0.0 }
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
#[cfg(feature = "egui")]
impl Into<egui::Vec2> for LogicalDelta {
  fn into(self) -> egui::Vec2 {
    // Note: loss of precision due to conversion from f64 into f32.
    egui::Vec2::new(self.x as f32, self.y as f32)
  }
}

impl Mul<Scale> for LogicalDelta {
  type Output = PhysicalDelta;
  #[inline]
  fn mul(self, rhs: Scale) -> Self::Output {
    // Note: rounds f64 into i64.
    PhysicalDelta::new((self.x * rhs).round() as _, (self.y * rhs).round() as _)
  }
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


/// Screen delta: combination of physical and logical delta.
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
    let physical = logical * scale.into();
    Self::new(physical, logical)
  }
  #[inline]
  pub fn from_physical_scale<P: Into<PhysicalDelta>, S: Into<Scale>>(physical: P, scale: S) -> Self {
    let physical = physical.into();
    let logical = physical / scale.into();
    Self::new(physical, logical)
  }
  #[inline]
  pub fn from_unscaled(x: i64, y: i64) -> Self {
    let physical = PhysicalDelta::new(x, y);
    let logical = physical / Scale::default();
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
