use std::collections::HashSet;

use crate::screen::{PhysicalDelta, PhysicalPosition};

#[derive(Clone, Debug, Default)]
pub struct RawInput {
  pub mouse_buttons: MouseButtons,
  pub mouse_pos: PhysicalPosition,
  pub mouse_pos_delta: PhysicalDelta,
  pub mouse_wheel_delta: MouseWheelDelta,
  pub keyboard_buttons: HashSet<VirtualKeyCode>,
  pub keyboard_buttons_pressed: HashSet<VirtualKeyCode>,
  pub keyboard_buttons_released: HashSet<VirtualKeyCode>,
  pub characters: Vec<char>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseButtons {
  pub left: bool,
  pub right: bool,
  pub middle: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseWheelDelta {
  pub x: f64,
  pub y: f64,
}

impl RawInput {
  pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons.contains(&key)
  }
  pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons_pressed.contains(&key)
  }
  pub fn is_key_released(&self, key: VirtualKeyCode) -> bool {
    self.keyboard_buttons_released.contains(&key)
  }


  pub fn remove_mouse_input(&mut self) {
    self.mouse_buttons.left = false;
    self.mouse_buttons.right = false;
    self.mouse_buttons.middle = false;
    self.mouse_pos_delta = PhysicalDelta::default();
    self.mouse_wheel_delta = MouseWheelDelta::default();
  }

  pub fn remove_keyboard_input(&mut self) {
    self.keyboard_buttons.clear();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters.clear();
  }

  pub fn clear_deltas(&mut self) {
    self.mouse_pos_delta = PhysicalDelta::default();
    self.mouse_wheel_delta = MouseWheelDelta::default();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters.clear();
  }
}

impl MouseWheelDelta {
  pub fn new(x: f64, y: f64) -> MouseWheelDelta { MouseWheelDelta { x, y } }
}

/// Symbolic name for a keyboard key.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum VirtualKeyCode {
  /// The '1' key over the letters.
  Key1,
  /// The '2' key over the letters.
  Key2,
  /// The '3' key over the letters.
  Key3,
  /// The '4' key over the letters.
  Key4,
  /// The '5' key over the letters.
  Key5,
  /// The '6' key over the letters.
  Key6,
  /// The '7' key over the letters.
  Key7,
  /// The '8' key over the letters.
  Key8,
  /// The '9' key over the letters.
  Key9,
  /// The '0' key over the 'O' and 'P' keys.
  Key0,

  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,

  /// The Escape key, next to F1.
  Escape,

  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,
  F16,
  F17,
  F18,
  F19,
  F20,
  F21,
  F22,
  F23,
  F24,

  /// Print Screen/SysRq.
  Snapshot,
  /// Scroll Lock.
  Scroll,
  /// Pause/Break key, next to Scroll lock.
  Pause,

  /// `Insert`, next to Backspace.
  Insert,
  Home,
  Delete,
  End,
  PageDown,
  PageUp,

  Left,
  Up,
  Right,
  Down,

  /// The Backspace key, right over Enter.
  // TODO: rename
  Back,
  /// The Enter key.
  Return,
  /// The space bar.
  Space,

  /// The "Compose" key on Linux.
  Compose,

  Caret,

  Numlock,
  Numpad0,
  Numpad1,
  Numpad2,
  Numpad3,
  Numpad4,
  Numpad5,
  Numpad6,
  Numpad7,
  Numpad8,
  Numpad9,
  NumpadAdd,
  NumpadDivide,
  NumpadDecimal,
  NumpadComma,
  NumpadEnter,
  NumpadEquals,
  NumpadMultiply,
  NumpadSubtract,

  AbntC1,
  AbntC2,
  Apostrophe,
  Apps,
  Asterisk,
  At,
  Ax,
  Backslash,
  Calculator,
  Capital,
  Colon,
  Comma,
  Convert,
  Equals,
  Grave,
  Kana,
  Kanji,
  LAlt,
  LBracket,
  LControl,
  LShift,
  LWin,
  Mail,
  MediaSelect,
  MediaStop,
  Minus,
  Mute,
  MyComputer,
  // also called "Next"
  NavigateForward,
  // also called "Prior"
  NavigateBackward,
  NextTrack,
  NoConvert,
  OEM102,
  Period,
  PlayPause,
  Plus,
  Power,
  PrevTrack,
  RAlt,
  RBracket,
  RControl,
  RShift,
  RWin,
  Semicolon,
  Slash,
  Sleep,
  Stop,
  Sysrq,
  Tab,
  Underline,
  Unlabeled,
  VolumeDown,
  VolumeUp,
  Wake,
  WebBack,
  WebFavorites,
  WebForward,
  WebHome,
  WebRefresh,
  WebSearch,
  WebStop,
  Yen,
  Copy,
  Paste,
  Cut,
}
