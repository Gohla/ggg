use std::sync::mpsc::Receiver;

use common::input::RawInput;
use common::screen::PhysicalDelta;

use crate::event_sys::{ElementState, OsInputEvent};

pub struct OsInputSys {
  input_event_rx: Receiver<OsInputEvent>,
  prev_state: Option<RawInput>,
}

impl OsInputSys {
  pub fn new(input_event_rx: Receiver<OsInputEvent>) -> OsInputSys {
    return OsInputSys {
      input_event_rx,
      prev_state: None,
    };
  }

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
        OsInputEvent::MouseInput { button, state } => {
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
        OsInputEvent::MouseMoved(position) => {
          input_state.mouse_position = position;
        }
        OsInputEvent::MouseWheelMovedPixels { horizontal_delta, vertical_delta } => {
          input_state.mouse_wheel_pixel_delta.horizontal += horizontal_delta;
          input_state.mouse_wheel_pixel_delta.vertical += vertical_delta;
        }
        OsInputEvent::MouseWheelMovedLines { horizontal_delta, vertical_delta } => {
          input_state.mouse_wheel_line_delta.horizontal += horizontal_delta;
          input_state.mouse_wheel_line_delta.vertical += vertical_delta;
        }
        OsInputEvent::KeyboardModifierChange { modifier, state } => {
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
        OsInputEvent::KeyboardInput { button, state } => {
          match state {
            ElementState::Pressed => {
              input_state.keyboard_buttons.insert(button);
              input_state.keyboard_buttons_pressed.insert(button);
            }
            ElementState::Released => {
              input_state.keyboard_buttons.remove(&button);
              input_state.keyboard_buttons_released.insert(button);
            }
          };
        }
        OsInputEvent::CharacterInput(c) => {
          input_state.characters_pressed.push(c);
        }
      }
    }

    input_state.mouse_position_delta = match self.prev_state {
      Some(ref prev_state) => PhysicalDelta::new(input_state.mouse_position.x - prev_state.mouse_position.x, input_state.mouse_position.y - prev_state.mouse_position.y),
      None => PhysicalDelta::default(),
    };

    self.prev_state = Some(input_state.clone());
    return input_state;
  }
}
