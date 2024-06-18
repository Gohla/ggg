use egui::{Button, PointerButton, Response, Ui};

use common::list::{Fold, Folder, Nil, Prepend};
use internal::{Reset, ResetFn};

pub trait UiResetButtonExt {
  fn reset_button(&mut self) -> ResetButton<Nil>;
}
impl UiResetButtonExt for Ui {
  #[inline]
  fn reset_button(&mut self) -> ResetButton<Nil> {
    ResetButton::new(self)
  }
}

pub struct ResetButton<'ui, R> {
  ui: &'ui mut Ui,
  can_reset: bool,
  resets: R,
}
impl<'ui> ResetButton<'ui, Nil> {
  pub fn new(ui: &'ui mut Ui) -> Self { Self { ui, can_reset: false, resets: Nil } }
}
impl<'ui, R: Prepend> ResetButton<'ui, R> {
  pub fn compare<T: PartialEq>(self, value: &mut T, reset: T) -> ResetButton<'ui, R::Output<Reset<T>>> {
    let can_reset = self.can_reset || *value != reset;
    ResetButton { ui: self.ui, can_reset, resets: self.resets.prepend(Reset::new(value, reset)) }
  }
}
impl<R: Fold<(), Folder<ResetFn>>> ResetButton<'_, R> {
  pub fn reset_on_click_by(mut self, button: PointerButton) {
    if self.add().clicked_by(button) {
      self.reset();
    }
  }
  pub fn reset_on_click(self) {
    self.reset_on_click_by(PointerButton::Primary);
  }
  pub fn reset_on_right_click(self) {
    self.reset_on_click_by(PointerButton::Secondary);
  }
  pub fn reset_on_middle_click(self) {
    self.reset_on_click_by(PointerButton::Middle);
  }

  pub fn reset_on_double_click_by(mut self, button: PointerButton) {
    if self.add().double_clicked_by(button) {
      self.reset();
    }
  }
  pub fn reset_on_double_click(self) {
    self.reset_on_double_click_by(PointerButton::Primary)
  }

  fn add(&mut self) -> Response {
    self.ui.add_enabled(self.can_reset, Button::new("↺"))
  }
  fn reset(self) {
    self.resets.fold((), Folder(ResetFn));
  }
}

mod internal {
  use common::list::FolderFn;

  pub struct ResetFn;
  impl<'a, T: PartialEq> FolderFn<(), Reset<'a, T>> for ResetFn {
    fn fold(&mut self, _: (), reset: Reset<'a, T>) {
      reset.reset();
    }
  }

  pub struct Reset<'a, T: PartialEq> {
    value: &'a mut T,
    reset: T,
  }
  impl<'a, T: PartialEq> Reset<'a, T> {
    pub fn new(value: &'a mut T, reset: T) -> Self {
      Self { value, reset }
    }
    pub fn reset(self) {
      *self.value = self.reset;
    }
  }
}
