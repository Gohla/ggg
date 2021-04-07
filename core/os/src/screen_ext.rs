use winit::dpi::{LogicalPosition as WinitLogicalPosition, LogicalSize as WinitLogicalSize, PhysicalPosition as WinitPhysicalPosition, PhysicalSize as WinitPhysicalSize, Pixel};

use common::screen::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};

pub trait LogicalSizeExt {
  fn into_winit(self) -> WinitLogicalSize<f64>;
}

impl LogicalSizeExt for LogicalSize {
  fn into_winit(self) -> WinitLogicalSize<f64> {
    let size: (f64, f64) = self.into();
    WinitLogicalSize::from(size)
  }
}

pub trait PhysicalSizeExt {
  fn into_winit(self) -> WinitPhysicalSize<u32>;
}

impl PhysicalSizeExt for PhysicalSize {
  /// Loss of precision from u64 into u32.
  fn into_winit(self) -> WinitPhysicalSize<u32> {
    let (width, height): (u64, u64) = self.into();
    WinitPhysicalSize::new(width as u32, height as u32)
  }
}

pub trait WinitLogicalSizeExt {
  fn into_common(self) -> LogicalSize;
}

impl<P: Pixel> WinitLogicalSizeExt for WinitLogicalSize<P> {
  fn into_common(self) -> LogicalSize {
    let size: (f64, f64) = self.into();
    LogicalSize::from(size)
  }
}

pub trait WinitPhysicalSizeExt {
  fn into_common(self) -> PhysicalSize;
}

impl<P: Pixel> WinitPhysicalSizeExt for WinitPhysicalSize<P> {
  fn into_common(self) -> PhysicalSize {
    let size: (u32, u32) = self.into();
    PhysicalSize::from(size)
  }
}


pub trait LogicalPositionExt {
  fn into_winit(self) -> WinitLogicalPosition<f64>;
}

impl LogicalPositionExt for LogicalPosition {
  fn into_winit(self) -> WinitLogicalPosition<f64> {
    let size: (f64, f64) = self.into();
    WinitLogicalPosition::from(size)
  }
}

pub trait PhysicalPositionExt {
  fn into_winit(self) -> WinitPhysicalPosition<i32>;
}

impl PhysicalPositionExt for PhysicalPosition {
  /// Loss of precision from i64 into i32.
  fn into_winit(self) -> WinitPhysicalPosition<i32> {
    let (x, y): (i64, i64) = self.into();
    WinitPhysicalPosition::new(x as i32, y as i32)
  }
}

pub trait WinitLogicalPositionExt {
  fn into_common(self) -> LogicalPosition;
}

impl<P: Pixel> WinitLogicalPositionExt for WinitLogicalPosition<P> {
  fn into_common(self) -> LogicalPosition {
    let size: (f64, f64) = self.into();
    LogicalPosition::from(size)
  }
}

pub trait WinitPhysicalPositionExt {
  fn into_common(self) -> PhysicalPosition;
}

impl<P: Pixel> WinitPhysicalPositionExt for WinitPhysicalPosition<P> {
  fn into_common(self) -> PhysicalPosition {
    let size: (i32, i32) = self.into();
    PhysicalPosition::from(size)
  }
}
