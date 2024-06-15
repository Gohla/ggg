use egui::{CursorIcon, Event, MouseWheelUnit, PlatformOutput, Pos2};

use common::input::{Key, KeyboardModifier, RawInput};
use os::clipboard::{get_clipboard, TextClipboard};
use os::open_url::open_url;
use os::window::Window;

pub struct GuiOs {
  clipboard: Box<dyn TextClipboard + Send + 'static>,
  input: egui::RawInput,
  cursor_icon: Option<CursorIcon>,
  cursor_in_window: bool,
}

// Creation

impl GuiOs {
  pub fn new() -> Self {
    Self {
      clipboard: get_clipboard(),
      input: egui::RawInput::default(),
      cursor_icon: None,
      cursor_in_window: false,
    }
  }

  #[inline]
  pub fn input(&mut self) -> egui::RawInput {
    std::mem::take(&mut self.input)
  }

  #[profiling::function]
  pub fn process_input(&mut self, input: &RawInput, process_keyboard: bool, process_mouse: bool) {
    if process_keyboard {
      // Keyboard modifiers
      self.input.modifiers.alt = input.is_keyboard_modifier_down(KeyboardModifier::Alternate);
      let is_control_down = input.is_keyboard_modifier_down(KeyboardModifier::Control);
      self.input.modifiers.ctrl = is_control_down;
      self.input.modifiers.shift = input.is_keyboard_modifier_down(KeyboardModifier::Shift);
      let is_super_down = input.is_keyboard_modifier_down(KeyboardModifier::Super);
      self.input.modifiers.mac_cmd = cfg!(target_os = "macos") && is_super_down;
      self.input.modifiers.command = if cfg!(target_os = "macos") { is_super_down } else { is_control_down };
    }
    let modifiers = self.input.modifiers;

    if process_mouse {
      // Mouse wheel delta
      if !input.mouse_wheel_pixel_delta.is_zero() {
        let delta = input.mouse_wheel_pixel_delta.logical.into();
        self.input.events.push(Event::MouseWheel { unit: MouseWheelUnit::Point, delta, modifiers });
      }
      if !input.mouse_wheel_line_delta.is_zero() {
        let delta = input.mouse_wheel_line_delta.into();
        self.input.events.push(Event::MouseWheel { unit: MouseWheelUnit::Line, delta, modifiers });
      }

      // Mouse movement
      let mouse_position: Pos2 = input.mouse_position.logical.into();
      if !input.mouse_position_delta.is_zero() {
        self.cursor_in_window = true;
        self.input.events.push(Event::PointerMoved(mouse_position))
      }

      // Mouse buttons
      for button in input.mouse_buttons_pressed() {
        if let Some(button) = button.into() {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: true, modifiers })
        }
      }
      for button in input.mouse_buttons_released() {
        if let Some(button) = button.into() {
          self.input.events.push(Event::PointerButton { pos: mouse_position, button, pressed: false, modifiers })
        }
      }
    }

    if process_keyboard {
      fn is_cut_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Cut
          || (modifiers.command && keycode == egui::Key::X)
          || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Delete)
      }
      fn is_copy_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Copy
          || (modifiers.command && keycode == egui::Key::C)
          || (cfg!(target_os = "windows") && modifiers.ctrl && keycode == egui::Key::Insert)
      }
      fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
        keycode == egui::Key::Paste
          || (modifiers.command && keycode == egui::Key::V)
          || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Insert)
      }
      /// Ignore special keys (backspace, delete, F1, …) that winit sends as characters. Also ignore '\r', '\n', '\t'
      /// since newlines are handled by the `Key::Enter` event.
      ///
      /// From: https://github.com/emilk/egui/blob/9f12432bcf8f8275f154cbbb8aabdb8958be9026/crates/egui-winit/src/lib.rs#L991-L1001
      fn is_printable_char(chr: char) -> bool {
        let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
          || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
          || '\u{100000}' <= chr && chr <= '\u{10fffd}';
        !is_in_private_use_area && !chr.is_ascii_control()
      }

      // Keyboard keys
      for Key { keyboard, semantic, text } in input.keys_pressed() {
        let physical_key: Option<egui::Key> = keyboard.and_then(|k| k.into());
        let logical_key: Option<egui::Key> = semantic.and_then(|s| s.into());
        let handle_text = if let Some(key) = logical_key.or(physical_key) {
          if is_cut_command(modifiers, key) {
            self.input.events.push(Event::Cut);
            false
          } else if is_copy_command(modifiers, key) {
            self.input.events.push(Event::Copy);
            false
          } else if is_paste_command(modifiers, key) {
            if let Some(contents) = self.clipboard.get() {
              let contents = contents.replace("\r\n", "\n");
              if !contents.is_empty() {
                self.input.events.push(Event::Paste(contents));
              }
            }
            false
          } else {
            self.input.events.push(Event::Key { key, physical_key, pressed: true, repeat: false, modifiers });
            true
          }
        } else {
          true
        };

        // On some platforms we get here when the user presses Cmd-C (copy), ctrl-W, etc. We need to ignore these
        // characters that are side effects of commands.
        let is_cmd = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;
        if handle_text && !is_cmd {
          if let Some(text) = text {
            if !text.is_empty() && text.chars().all(is_printable_char) {
              self.input.events.push(Event::Text(text.to_string()))
            }
          }
        }
      }
      for Key { keyboard, semantic, .. } in input.keys_released() {
        let physical_key: Option<egui::Key> = keyboard.and_then(|k| k.into());
        let logical_key: Option<egui::Key> = semantic.and_then(|s| s.into());
        if let Some(key) = logical_key.or(physical_key) {
          self.input.events.push(Event::Key { key, physical_key, pressed: false, repeat: false, modifiers });
        }
        // Note: not handling text as egui doesn't need it for released keys.
      }
    }
  }

  pub fn process_window_cursor_event(&mut self, cursor_in_window: bool) {
    self.cursor_in_window = cursor_in_window;
    if !cursor_in_window {
      self.input.events.push(Event::PointerGone);
    }
  }

  pub fn process_window_focus_event(&mut self, focus: bool) {
    self.input.focused = focus;
    self.input.events.push(Event::WindowFocused(focus));
  }

  #[profiling::function]
  pub fn process_platform_output(&mut self, window: &Window, platform_output: PlatformOutput) {
    self.set_cursor_icon(window, platform_output.cursor_icon);

    if let Some(url) = platform_output.open_url {
      open_url(&url.url, url.new_tab);
    }

    if !platform_output.copied_text.is_empty() {
      self.clipboard.set(&platform_output.copied_text)
    }
  }

  fn set_cursor_icon(&mut self, window: &Window, cursor_icon: CursorIcon) {
    if self.cursor_icon == Some(cursor_icon) {
      return;
    }

    if self.cursor_in_window {
      self.cursor_icon = Some(cursor_icon);
      window.set_option_cursor(cursor_icon.into())
    } else {
      self.cursor_icon = None;
    }
  }
}

