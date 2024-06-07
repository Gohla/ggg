use std::sync::mpsc::Receiver;

use common::input::RawInput;
use common::screen::ScreenDelta;

use crate::event::{ElementState, InputEvent};

pub struct InputSys {
  input_event_rx: Receiver<InputEvent>,
  prev_state: Option<RawInput>,
}

impl InputSys {
  pub fn new(input_event_rx: Receiver<InputEvent>) -> Self {
    Self { input_event_rx, prev_state: None }
  }

  #[profiling::function]
  pub fn update(&mut self) -> RawInput {
    let mut input_state = if let Some(ref prev_state) = self.prev_state {
      let mut prev_state = prev_state.clone();
      prev_state.clear_deltas();
      prev_state
    } else {
      RawInput::default()
    };

    for event in self.input_event_rx.try_iter() {
      match event {
        InputEvent::MouseButton { button, state } => {
          match state {
            ElementState::Pressed => {
              input_state.mouse_buttons.insert(button);
              input_state.mouse_buttons_pressed.insert(button);
            }
            ElementState::Released => {
              input_state.mouse_buttons.remove(&button);
              input_state.mouse_buttons_released.insert(button);
            }
          };
        }
        InputEvent::MousePosition(position) => {
          input_state.mouse_position = position;
        }
        InputEvent::MouseWheelPixel(screen_delta) => {
          input_state.mouse_wheel_pixel_delta += screen_delta;
        }
        InputEvent::MouseWheelLine(line_delta) => {
          input_state.mouse_wheel_line_delta += line_delta;
        }
        InputEvent::KeyboardModifier { modifier, state } => {
          match state {
            ElementState::Pressed => {
              input_state.keyboard_modifiers.insert(modifier);
              input_state.keyboard_modifiers_pressed.insert(modifier);
            }
            ElementState::Released => {
              input_state.keyboard_modifiers.remove(&modifier);
              input_state.keyboard_modifiers_released.insert(modifier);
            }
          };
        }
        InputEvent::KeyboardKey { keyboard_key, semantic_key, text, state } => {
          if let Some(keyboard_key) = keyboard_key {
            if state.is_pressed() {
              input_state.keyboard_keys.insert(keyboard_key);
              input_state.keyboard_keys_pressed.insert(keyboard_key);
            } else {
              input_state.keyboard_keys.remove(&keyboard_key);
              input_state.keyboard_keys_released.insert(keyboard_key);
            }
          }
          if let Some(semantic_key) = semantic_key {
            if state.is_pressed() {
              input_state.semantic_keys.insert(semantic_key);
              input_state.semantic_keys_pressed.insert(semantic_key);
            } else {
              input_state.semantic_keys.remove(&semantic_key);
              input_state.semantic_keys_released.insert(semantic_key);
            }
          }
          if let Some(text) = text {
            if state.is_pressed() {
              input_state.text_inserted.push_str(&text);
            }
          }
        }
      }
    }

    input_state.mouse_position_delta = match self.prev_state {
      Some(ref prev_state) => input_state.mouse_position - prev_state.mouse_position,
      None => ScreenDelta::default(),
    };

    self.prev_state = Some(input_state.clone());
    return input_state;
  }
}
