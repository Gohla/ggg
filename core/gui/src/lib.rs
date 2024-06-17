use std::ops::Deref;

use egui::{Context, Rect, WidgetText, Window};

pub mod widget;
pub mod reset;

/// Handles and data for creating GUIs.
pub struct Gui {
  /// Handle for creating top-level GUI elements with egui.
  pub context: Context,
  /// Area under the title bar. Constrain windows to this area so that they can't overlap with the menu bar.
  pub area_under_title_bar: Rect,
}
impl Deref for Gui {
  type Target = Context;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.context }
}

impl Gui {
  #[inline]
  pub fn window(&self, title: impl Into<WidgetText>) -> Window {
    Window::new(title)
      .constrain_to(self.area_under_title_bar)
  }
}
