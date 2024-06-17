use common::list::{Fold, Folder, Nil, Prepend, ToRef};
use internal::{CanResetFn, Reset, ResetFn};

pub struct ResetChain<L>(L);
impl ResetChain<Nil> {
  pub fn new() -> Self { Self(Nil) }
}
impl<L: Prepend> ResetChain<L> {
  #[inline]
  pub fn add<T: PartialEq>(self, value: &mut T, reset: T) -> ResetChain<L::Output<Reset<T>>> {
    ResetChain(self.0.prepend(Reset::new(value, reset)))
  }
}
// impl<L> ResetChain<L> where
//   L: Fold<(), Folder<ResetFn>>,
// {
//   pub fn reset_button_double_click<'a>(self, ui: &mut Ui) where L: ToRef<Ref<'a>: Fold<bool, Folder<CanResetFn>>>, L: 'a {
//     let can_reset = self.0.to_ref().fold(false, Folder(CanResetFn));
//     if ui.add_enabled(can_reset, Button::new("↺")).double_clicked() {
//       self.0.fold((), Folder(ResetFn));
//     }
//   }
// }
impl<L> ResetChain<L> {
  pub fn can_reset<'a>(&'a self) -> bool where
    L: ToRef<Ref<'a>: Fold<bool, Folder<CanResetFn>>>
  {
    self.0.to_ref().fold(false, Folder(CanResetFn))
  }
}
impl<L: Fold<(), Folder<ResetFn>>> ResetChain<L> {
  pub fn reset(self) {
    self.0.fold((), Folder(ResetFn));
  }
}

mod internal {
  use common::list::FolderFn;

  pub struct CanResetFn;
  impl<'a, 'b, T: PartialEq> FolderFn<bool, &'b Reset<'a, T>> for CanResetFn {
    fn fold(&mut self, can_reset: bool, reset: &'b Reset<'a, T>) -> bool {
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
