use std::sync::Arc;

use thiserror::Error;
use winit::error::OsError;
use winit::window::WindowAttributes;
pub use winit::window::WindowId;

use common::cursor;
use common::screen::{LogicalSize, Scale, ScreenSize};

use crate::context::Context;

#[derive(Debug)]
pub struct Window {
  window: Arc<winit::window::Window>,
}

#[derive(Debug, Error)]
#[error("Could not create window: {0}")]
pub struct WindowCreateError(#[from] OsError);

impl Window {
  pub fn new<S: Into<String>>(
    os_context: &Context,
    inner_size: LogicalSize,
    min_inner_size: LogicalSize,
    title: S,
  ) -> Result<Self, WindowCreateError> {
    let window_attributes = WindowAttributes::default()
      .with_inner_size(inner_size)
      .with_min_inner_size(min_inner_size)
      .with_title(title);
    #[allow(deprecated)]
      let window = os_context.event_loop.create_window(window_attributes)?;

    #[cfg(target_arch = "wasm32")] {
      use winit::platform::web::WindowExtWebSys;
      let canvas = window.canvas();
      let web_window = web_sys::window().expect("no global `window` exists");
      let document = web_window.document().expect("should have a document on window");
      let body = document.body().unwrap();
      body.style().set_property("background-color", "black").ok();
      body.style().set_property("margin", "0px").ok();
      body.style().set_property("overflow", "hidden").ok();
      body.append_child(&canvas)
        .expect("Append canvas to HTML body");

      let window_clone = window.clone();
      let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let size = get_browser_inner_size();
        window_clone.set_inner_size(size.into_winit())
      }) as Box<dyn FnMut(_)>);
      use wasm_bindgen::JsCast;
      web_window
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        .unwrap();
      closure.forget();
    }

    let window = Arc::new(window);
    Ok(Self { window })
  }

  #[inline]
  pub fn as_winit_window(&self) -> &winit::window::Window { &self.window }
  #[inline]
  pub fn cloned_winit_window(&self) -> Arc<winit::window::Window> { self.window.clone() }

  #[inline]
  pub fn id(&self) -> WindowId {
    self.window.id()
  }
  #[inline]
  pub fn scale_factor(&self) -> Scale {
    self.window.scale_factor().into()
  }
  #[inline]
  pub fn inner_size(&self) -> ScreenSize {
    ScreenSize::from_physical_scale(self.window.inner_size(), self.scale_factor())
  }
  #[inline]
  pub fn outer_size(&self) -> ScreenSize {
    ScreenSize::from_physical_scale(self.window.outer_size(), self.scale_factor())
  }

  /// Makes the cursor visible and sets it to `icon`.
  #[inline]
  pub fn set_cursor(&self, icon: cursor::Icon) {
    self.window.set_cursor_visible(true);
    self.window.set_cursor(icon)
  }
  /// If `option_icon` is `Some(icon)`, makes the cursor visible and sets it to `icon`. Otherwise, makes the cursor
  /// invisible.
  #[inline]
  pub fn set_option_cursor(&self, option_icon: cursor::OptionIcon) {
    if let Some(icon) = option_icon.into_option() {
      self.window.set_cursor_visible(true);
      self.window.set_cursor(icon)
    } else {
      self.window.set_cursor_visible(false);
    }
  }
}

#[cfg(target_arch = "wasm32")]
pub fn get_browser_inner_size() -> LogicalSize {
  let window = web_sys::window().expect("no global `window` exists");
  let default_width = 1280.0;
  let default_height = 720.0;
  let client_width = window.inner_width().map_or(default_width, |v| v.as_f64().unwrap_or(default_width));
  let client_height = window.inner_height().map_or(default_height, |v| v.as_f64().unwrap_or(default_height));
  LogicalSize::new(client_width, client_height)
}
