use std::collections::HashSet;

use crate::screen::{ScreenDelta, ScreenPosition};

#[derive(Default, Clone, Debug)]
pub struct RawInput {
  pub mouse_buttons: HashSet<MouseButton>,
  pub mouse_buttons_pressed: HashSet<MouseButton>,
  pub mouse_buttons_released: HashSet<MouseButton>,

  pub mouse_position: ScreenPosition,
  pub mouse_position_delta: ScreenDelta,
  pub mouse_wheel_pixel_delta: ScreenDelta,
  pub mouse_wheel_line_delta: MouseWheelDelta,

  pub keyboard_modifiers: HashSet<KeyboardModifier>,
  pub keyboard_modifiers_pressed: HashSet<KeyboardModifier>,
  pub keyboard_modifiers_released: HashSet<KeyboardModifier>,

  pub keyboard_buttons: HashSet<KeyboardButton>,
  pub keyboard_buttons_pressed: HashSet<KeyboardButton>,
  pub keyboard_buttons_released: HashSet<KeyboardButton>,

  pub characters_pressed: Vec<char>,
}

impl RawInput {
  pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
    self.mouse_buttons.contains(&button)
  }
  pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
    self.mouse_buttons_pressed.contains(&button)
  }
  pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
    self.mouse_buttons_released.contains(&button)
  }

  pub fn is_keyboard_modifier_down(&self, modifier: KeyboardModifier) -> bool {
    self.keyboard_modifiers.contains(&modifier)
  }
  pub fn is_keyboard_modifier_pressed(&self, modifier: KeyboardModifier) -> bool {
    self.keyboard_modifiers_pressed.contains(&modifier)
  }
  pub fn is_keyboard_modifier_released(&self, modifier: KeyboardModifier) -> bool {
    self.keyboard_modifiers_released.contains(&modifier)
  }

  pub fn is_keyboard_button_down(&self, button: KeyboardButton) -> bool {
    self.keyboard_buttons.contains(&button)
  }
  pub fn is_keyboard_button_pressed(&self, button: KeyboardButton) -> bool {
    self.keyboard_buttons_pressed.contains(&button)
  }
  pub fn is_keyboard_button_released(&self, button: KeyboardButton) -> bool {
    self.keyboard_buttons_released.contains(&button)
  }


  pub fn remove_mouse_input(&mut self) {
    self.mouse_buttons.clear();
    self.mouse_buttons_pressed.clear();
    self.mouse_buttons_released.clear();
    self.mouse_position_delta = ScreenDelta::default();
    self.mouse_wheel_pixel_delta = ScreenDelta::default();
    self.mouse_wheel_line_delta = MouseWheelDelta::default();
  }

  pub fn remove_keyboard_input(&mut self) {
    self.keyboard_modifiers.clear();
    self.keyboard_modifiers_pressed.clear();
    self.keyboard_modifiers_released.clear();
    self.keyboard_buttons.clear();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters_pressed.clear();
  }

  pub fn clear_deltas(&mut self) {
    self.mouse_buttons_pressed.clear();
    self.mouse_buttons_released.clear();
    self.mouse_position_delta = ScreenDelta::default();
    self.mouse_wheel_pixel_delta = ScreenDelta::default();
    self.mouse_wheel_line_delta = MouseWheelDelta::default();
    self.keyboard_modifiers_pressed.clear();
    self.keyboard_modifiers_released.clear();
    self.keyboard_buttons_pressed.clear();
    self.keyboard_buttons_released.clear();
    self.characters_pressed.clear();
  }
}


/// Describes a button of a mouse controller.
///
/// ## Platform-specific
///
/// **macOS:** `Back` and `Forward` might not work with all hardware.
/// **Orbital:** `Back` and `Forward` are unsupported due to orbital not supporting them.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum MouseButton {
  Left,
  Right,
  Middle,
  Back,
  Forward,
  Other(u16),
}

#[cfg(feature = "winit")]
impl From<winit::event::MouseButton> for MouseButton {
  #[inline]
  fn from(button: winit::event::MouseButton) -> Self {
    use winit::event::MouseButton::*;
    match button {
      Left => Self::Left,
      Right => Self::Right,
      Middle => Self::Middle,
      Back => Self::Back,
      Forward => Self::Forward,
      Other(b) => Self::Other(b),
    }
  }
}


#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct MouseWheelDelta {
  pub horizontal: f64,
  pub vertical: f64,
}

impl MouseWheelDelta {
  #[inline]
  pub fn new(x: f64, y: f64) -> Self { Self { horizontal: x, vertical: y } }

  #[inline]
  pub fn is_zero(&self) -> bool { self.horizontal != 0.0 && self.vertical == 0.0 }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum KeyboardModifier {
  Shift,
  Control,
  Alternate,
  Meta,
}

/// Symbolic name for a keyboard key.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum KeyboardButton {
  /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
  /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
  /// (hankaku/zenkaku/kanji) key on Japanese keyboards
  Backquote,
  /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
  /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
  /// 104- and 106-key layouts.
  /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
  Backslash,
  /// <kbd>[</kbd> on a US keyboard.
  BracketLeft,
  /// <kbd>]</kbd> on a US keyboard.
  BracketRight,
  /// <kbd>,</kbd> on a US keyboard.
  Comma,
  /// <kbd>0</kbd> on a US keyboard.
  Digit0,
  /// <kbd>1</kbd> on a US keyboard.
  Digit1,
  /// <kbd>2</kbd> on a US keyboard.
  Digit2,
  /// <kbd>3</kbd> on a US keyboard.
  Digit3,
  /// <kbd>4</kbd> on a US keyboard.
  Digit4,
  /// <kbd>5</kbd> on a US keyboard.
  Digit5,
  /// <kbd>6</kbd> on a US keyboard.
  Digit6,
  /// <kbd>7</kbd> on a US keyboard.
  Digit7,
  /// <kbd>8</kbd> on a US keyboard.
  Digit8,
  /// <kbd>9</kbd> on a US keyboard.
  Digit9,
  /// <kbd>=</kbd> on a US keyboard.
  Equal,
  /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
  /// Labeled <kbd>\\</kbd> on a UK keyboard.
  IntlBackslash,
  /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
  /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
  IntlRo,
  /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
  /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
  /// Russian keyboard.
  IntlYen,
  /// <kbd>a</kbd> on a US keyboard.
  /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
  KeyA,
  /// <kbd>b</kbd> on a US keyboard.
  KeyB,
  /// <kbd>c</kbd> on a US keyboard.
  KeyC,
  /// <kbd>d</kbd> on a US keyboard.
  KeyD,
  /// <kbd>e</kbd> on a US keyboard.
  KeyE,
  /// <kbd>f</kbd> on a US keyboard.
  KeyF,
  /// <kbd>g</kbd> on a US keyboard.
  KeyG,
  /// <kbd>h</kbd> on a US keyboard.
  KeyH,
  /// <kbd>i</kbd> on a US keyboard.
  KeyI,
  /// <kbd>j</kbd> on a US keyboard.
  KeyJ,
  /// <kbd>k</kbd> on a US keyboard.
  KeyK,
  /// <kbd>l</kbd> on a US keyboard.
  KeyL,
  /// <kbd>m</kbd> on a US keyboard.
  KeyM,
  /// <kbd>n</kbd> on a US keyboard.
  KeyN,
  /// <kbd>o</kbd> on a US keyboard.
  KeyO,
  /// <kbd>p</kbd> on a US keyboard.
  KeyP,
  /// <kbd>q</kbd> on a US keyboard.
  /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
  KeyQ,
  /// <kbd>r</kbd> on a US keyboard.
  KeyR,
  /// <kbd>s</kbd> on a US keyboard.
  KeyS,
  /// <kbd>t</kbd> on a US keyboard.
  KeyT,
  /// <kbd>u</kbd> on a US keyboard.
  KeyU,
  /// <kbd>v</kbd> on a US keyboard.
  KeyV,
  /// <kbd>w</kbd> on a US keyboard.
  /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
  KeyW,
  /// <kbd>x</kbd> on a US keyboard.
  KeyX,
  /// <kbd>y</kbd> on a US keyboard.
  /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
  KeyY,
  /// <kbd>z</kbd> on a US keyboard.
  /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
  /// QWERTZ (e.g., German) keyboard.
  KeyZ,
  /// <kbd>-</kbd> on a US keyboard.
  Minus,
  /// <kbd>.</kbd> on a US keyboard.
  Period,
  /// <kbd>'</kbd> on a US keyboard.
  Quote,
  /// <kbd>;</kbd> on a US keyboard.
  Semicolon,
  /// <kbd>/</kbd> on a US keyboard.
  Slash,
  /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
  AltLeft,
  /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
  /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
  AltRight,
  /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
  /// Labeled <kbd>Delete</kbd> on Apple keyboards.
  Backspace,
  /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
  CapsLock,
  /// The application context menu key, which is typically found between the right
  /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
  ContextMenu,
  /// <kbd>Control</kbd> or <kbd>⌃</kbd>
  ControlLeft,
  /// <kbd>Control</kbd> or <kbd>⌃</kbd>
  ControlRight,
  /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
  Enter,
  /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
  SuperLeft,
  /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
  SuperRight,
  /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
  ShiftLeft,
  /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
  ShiftRight,
  /// <kbd> </kbd> (space)
  Space,
  /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
  Tab,
  /// Japanese: <kbd>変</kbd> (henkan)
  Convert,
  /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
  KanaMode,
  /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
  ///
  /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
  Lang1,
  /// Korean: Hanja <kbd>한</kbd> (hanja)
  ///
  /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
  Lang2,
  /// Japanese (word-processing keyboard): Katakana
  Lang3,
  /// Japanese (word-processing keyboard): Hiragana
  Lang4,
  /// Japanese (word-processing keyboard): Zenkaku/Hankaku
  Lang5,
  /// Japanese: <kbd>無変換</kbd> (muhenkan)
  NonConvert,
  /// <kbd>⌦</kbd>. The forward delete key.
  /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
  /// the keyboard is encoded as [`Backspace`].
  ///
  /// [`Backspace`]: Self::Backspace
  Delete,
  /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
  End,
  /// <kbd>Help</kbd>. Not present on standard PC keyboards.
  Help,
  /// <kbd>Home</kbd> or <kbd>↖</kbd>
  Home,
  /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
  Insert,
  /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
  PageDown,
  /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
  PageUp,
  /// <kbd>↓</kbd>
  ArrowDown,
  /// <kbd>←</kbd>
  ArrowLeft,
  /// <kbd>→</kbd>
  ArrowRight,
  /// <kbd>↑</kbd>
  ArrowUp,
  /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
  NumLock,
  /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
  Numpad0,
  /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
  Numpad1,
  /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
  Numpad2,
  /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
  Numpad3,
  /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
  Numpad4,
  /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
  Numpad5,
  /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
  Numpad6,
  /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
  /// or remote control
  Numpad7,
  /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
  Numpad8,
  /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
  /// or remote control
  Numpad9,
  /// <kbd>+</kbd>
  NumpadAdd,
  /// Found on the Microsoft Natural Keyboard.
  NumpadBackspace,
  /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
  /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
  /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
  ///
  /// [`NumLock`]: Self::NumLock
  NumpadClear,
  /// <kbd>C</kbd> (Clear Entry)
  NumpadClearEntry,
  /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
  /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
  NumpadComma,
  /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
  /// Brazil), this key may generate a <kbd>,</kbd>.
  NumpadDecimal,
  /// <kbd>/</kbd>
  NumpadDivide,
  NumpadEnter,
  /// <kbd>=</kbd>
  NumpadEqual,
  /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
  /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
  NumpadHash,
  /// <kbd>M</kbd> Add current entry to the value stored in memory.
  NumpadMemoryAdd,
  /// <kbd>M</kbd> Clear the value stored in memory.
  NumpadMemoryClear,
  /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
  NumpadMemoryRecall,
  /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
  NumpadMemoryStore,
  /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
  NumpadMemorySubtract,
  /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
  /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
  ///
  /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
  NumpadMultiply,
  /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
  NumpadParenLeft,
  /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
  NumpadParenRight,
  /// <kbd>*</kbd> on a phone or remote control device.
  ///
  /// This key is typically found below the <kbd>7</kbd> key and to the left of
  /// the <kbd>0</kbd> key.
  ///
  /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
  /// numeric keypads.
  NumpadStar,
  /// <kbd>-</kbd>
  NumpadSubtract,
  /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
  Escape,
  /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
  Fn,
  /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
  /// Natural Keyboard.
  FnLock,
  /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
  PrintScreen,
  /// <kbd>Scroll Lock</kbd>
  ScrollLock,
  /// <kbd>Pause Break</kbd>
  Pause,
  /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
  ///
  /// This also the "back" button (triangle) on Android.
  BrowserBack,
  BrowserFavorites,
  /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
  BrowserForward,
  /// The "home" button on Android.
  BrowserHome,
  BrowserRefresh,
  BrowserSearch,
  BrowserStop,
  /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
  /// keyboards.
  Eject,
  /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
  LaunchApp1,
  /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
  LaunchApp2,
  LaunchMail,
  MediaPlayPause,
  MediaSelect,
  MediaStop,
  MediaTrackNext,
  MediaTrackPrevious,
  /// This key is placed in the function section on some Apple keyboards, replacing the
  /// <kbd>Eject</kbd> key.
  Power,
  Sleep,
  AudioVolumeDown,
  AudioVolumeMute,
  AudioVolumeUp,
  WakeUp,
  // Legacy modifier key. Also called "Super" in certain places.
  Meta,
  // Legacy modifier key.
  Hyper,
  Turbo,
  Abort,
  Resume,
  Suspend,
  /// Found on Sun’s USB keyboard.
  Again,
  /// Found on Sun’s USB keyboard.
  Copy,
  /// Found on Sun’s USB keyboard.
  Cut,
  /// Found on Sun’s USB keyboard.
  Find,
  /// Found on Sun’s USB keyboard.
  Open,
  /// Found on Sun’s USB keyboard.
  Paste,
  /// Found on Sun’s USB keyboard.
  Props,
  /// Found on Sun’s USB keyboard.
  Select,
  /// Found on Sun’s USB keyboard.
  Undo,
  /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
  Hiragana,
  /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
  Katakana,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F1,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F2,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F3,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F4,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F5,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F6,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F7,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F8,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F9,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F10,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F11,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F12,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F13,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F14,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F15,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F16,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F17,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F18,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F19,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F20,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F21,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F22,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F23,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F24,
  /// General-purpose function key.
  F25,
  /// General-purpose function key.
  F26,
  /// General-purpose function key.
  F27,
  /// General-purpose function key.
  F28,
  /// General-purpose function key.
  F29,
  /// General-purpose function key.
  F30,
  /// General-purpose function key.
  F31,
  /// General-purpose function key.
  F32,
  /// General-purpose function key.
  F33,
  /// General-purpose function key.
  F34,
  /// General-purpose function key.
  F35,
}

#[cfg(feature = "winit")]
impl From<winit::keyboard::KeyCode> for KeyboardButton {
  #[inline]
  fn from(key_code: winit::keyboard::KeyCode) -> Self {
    unsafe { std::mem::transmute(key_code) }
  }
}
