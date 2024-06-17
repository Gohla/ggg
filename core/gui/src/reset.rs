use egui::{Button, PointerButton, Response, Ui};

use common::list::{Fold, Folder, FoldRef, Nil, Prepend};
use internal::{CanResetFn, Reset, ResetFn};

pub trait UiResetButtonExt {
  fn reset_button_new(&mut self) -> ResetButton<Nil>;
}
impl UiResetButtonExt for Ui {
  fn reset_button_new(&mut self) -> ResetButton<Nil> {
    ResetButton::new(self)
  }
}

pub struct ResetButton<'ui, L> {
  list: L,
  ui: &'ui mut Ui,
}
impl<'ui> ResetButton<'ui, Nil> {
  pub fn new(ui: &'ui mut Ui) -> Self { Self { list: Nil, ui } }
}
impl<'ui, L: Prepend> ResetButton<'ui, L> {
  #[inline]
  pub fn check<T: PartialEq>(self, value: &mut T, reset: T) -> ResetButton<'ui, L::Output<Reset<T>>> {
    ResetButton { list: self.list.prepend(Reset::new(value, reset)), ui: self.ui }
  }
}
impl<L> ResetButton<'_, L> where
  L: FoldRef<bool, Folder<CanResetFn>>,
  L: Fold<(), Folder<ResetFn>>,
{
  pub fn can_reset(&self) -> bool {
    self.list.fold_ref(false, Folder(CanResetFn))
  }
  pub fn reset(self) {
    self.list.fold((), Folder(ResetFn));
  }

  pub fn add(&mut self) -> Response {
    let can_reset = self.can_reset();
    self.ui.add_enabled(can_reset, Button::new("↺"))
  }

  pub fn on_click_by(mut self, button: PointerButton) {
    if self.add().clicked_by(button) {
      self.reset();
    }
  }
  pub fn on_click(self) {
    self.on_click_by(PointerButton::Primary);
  }
  pub fn on_middle_click(self) {
    self.on_click_by(PointerButton::Middle);
  }

  pub fn on_double_click_by(mut self, button: PointerButton) {
    if self.add().double_clicked_by(button) {
      self.reset();
    }
  }
  pub fn on_double_click(self) {
    self.on_double_click_by(PointerButton::Primary)
  }
}

mod internal {
  use common::list::FolderFn;

  pub struct CanResetFn;
  impl<'a, T: PartialEq> FolderFn<bool, &Reset<'a, T>> for CanResetFn {
    fn fold(&mut self, can_reset: bool, reset: &Reset<'a, T>) -> bool {
      can_reset || reset.can_reset()
    }
  }
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
    #[inline]
    pub fn new(value: &'a mut T, reset: T) -> Self {
      Self { value, reset }
    }
    #[inline]
    pub fn can_reset(&'a self) -> bool {
      *self.value != self.reset
    }
    #[inline]
    pub fn reset(self) {
      *self.value = self.reset;
    }
  }
}
