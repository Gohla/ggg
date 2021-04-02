use std::ops::Deref;

use thiserror::Error;
use winit::error::OsError;
use winit::window::{Window as WinitWindow, WindowBuilder};

use math::screen::{LogicalSize, ScreenSize};

use crate::context::OsContext;
use crate::screen_ext::*;

pub struct Window {
  window: WinitWindow,
}

#[derive(Debug, Error)]
#[error("Could not create window")]
pub struct WindowCreateError(#[from] OsError);

impl Window {
  pub fn new<S: Into<String>>(
    os_context: &OsContext,
    inner_size: LogicalSize,
    min_inner_size: LogicalSize,
    title: S,
  ) -> Result<Self, WindowCreateError> {
    let window = WindowBuilder::new()
      .with_inner_size(inner_size.into_winit())
      .with_min_inner_size(min_inner_size.into_winit())
      .with_title(title)
      .build(&os_context.event_loop)?;
    Ok(Self { window })
  }


  #[inline]
  pub fn get_inner(&self) -> &WinitWindow {
    &self.window
  }


  pub fn get_inner_size(&self) -> ScreenSize {
    let physical_size: (u32, u32) = self.window.inner_size().into();
    let scale = self.window.scale_factor();
    ScreenSize::from_physical_scale(physical_size, scale)
  }
}

impl Deref for Window {
  type Target = WinitWindow;

  #[inline]
  fn deref(&self) -> &Self::Target { self.get_inner() }
}
