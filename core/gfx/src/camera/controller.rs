use std::f32::consts::{FRAC_PI_2, PI};

use ultraviolet::{Rotor3, Vec3};

use common::input::{KeyboardKey, MouseButton, RawInput};
use common::screen::ScreenDelta;
use common::time::Offset;

/// Camera controller settings.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraControllerSettings {
  pub control_type: ControlType,
  pub arcball: ArcballSettings,
}
impl CameraControllerSettings {
  #[inline]
  pub fn new(control_type: ControlType, arcball: ArcballSettings) -> Self { Self { control_type, arcball } }
}

/// Types of camera controllers.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ControlType {
  #[default]
  Arcball,
}

/// Camera controller arcball settings.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ArcballSettings {
  pub mouse_movement_panning_speed: f32,
  pub keyboard_panning_speed: f32,
  pub debug_gui_panning_speed: f32,

  pub mouse_scroll_distance_speed: f32,
  pub debug_gui_distance_speed: f32,

  pub mouse_movement_rotation_speed: f32,
  pub debug_gui_rotation_speed: f32,
}
impl Default for ArcballSettings {
  #[inline]
  fn default() -> Self {
    Self {
      mouse_movement_panning_speed: 0.0025,
      keyboard_panning_speed: 1.0,
      debug_gui_panning_speed: 1.0,

      mouse_scroll_distance_speed: 0.1,
      debug_gui_distance_speed: 0.1,

      mouse_movement_rotation_speed: 0.0025,
      debug_gui_rotation_speed: 0.01,
    }
  }
}


/// Camera controller input
#[derive(Default, Copy, Clone, Debug)]
pub struct CameraControllerInput {
  primary_mouse_button_down: bool,
  secondary_mouse_button_down: bool,
  mouse_position_delta: ScreenDelta,
  mouse_wheel_scroll_delta: f32,
  forward_key_down: bool,
  left_key_down: bool,
  backward_key_down: bool,
  right_key_down: bool,
  up_key_down: bool,
  down_key_down: bool,
}
impl From<&RawInput> for CameraControllerInput {
  fn from(input: &RawInput) -> Self {
    Self {
      primary_mouse_button_down: input.is_mouse_button_down(MouseButton::Left),
      secondary_mouse_button_down: input.is_mouse_button_down(MouseButton::Right),
      mouse_position_delta: input.mouse_position_delta,
      mouse_wheel_scroll_delta: input.mouse_wheel_pixel_delta.physical.y as f32 + input.mouse_wheel_line_delta.y as f32,
      forward_key_down: input.is_keyboard_key_down(KeyboardKey::KeyW),
      left_key_down: input.is_keyboard_key_down(KeyboardKey::KeyA),
      backward_key_down: input.is_keyboard_key_down(KeyboardKey::KeyS),
      right_key_down: input.is_keyboard_key_down(KeyboardKey::KeyD),
      up_key_down: input.is_keyboard_key_down(KeyboardKey::Space),
      down_key_down: input.is_keyboard_key_down(KeyboardKey::KeyC),
    }
  }
}


/// Camera controller state
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraControllerState {
  pub target: Vec3,
  pub arcball: ArcballState,
}
impl Default for CameraControllerState {
  #[inline]
  fn default() -> Self {
    Self {
      target: Vec3::zero(),
      arcball: ArcballState::default(),
    }
  }
}

/// Arcball data for camera controller.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ArcballState {
  pub distance: f32,
  pub rotation_around_x: f32,
  pub rotation_around_y: f32,
}
impl Default for ArcballState {
  #[inline]
  fn default() -> Self {
    Self {
      distance: 1.0,
      rotation_around_x: 0.0,
      rotation_around_y: 0.0,
    }
  }
}


/// Camera controller
#[derive(Default, Copy, Clone, Debug)]
pub struct CameraController {
  position: Vec3,
  target: Vec3,
}
impl CameraController {
  #[inline]
  pub fn new() -> Self { Self::default() }

  /// Gets the position of this controller. That is, the eye of the camera.
  #[inline]
  pub fn position(&self) -> Vec3 { self.position }
  /// Gets the target of this controller. That is, the target the eye of the camera looks at.
  #[inline]
  pub fn target(&self) -> Vec3 { self.target }

  /// Update the position and target.
  pub fn update(
    &mut self,
    state: &mut CameraControllerState,
    settings: &CameraControllerSettings,
    input: &CameraControllerInput,
    frame_duration: Offset,
  ) {
    match settings.control_type {
      ControlType::Arcball => {
        // Rotation
        // Scrolling up should zoom in, decreasing distance, so multiply by -1.0.
        state.arcball.distance += input.mouse_wheel_scroll_delta * settings.arcball.mouse_scroll_distance_speed * -1.0;
        if state.arcball.distance < 0.1 { state.arcball.distance = 0.1; }
        if input.primary_mouse_button_down && !input.secondary_mouse_button_down {
          state.arcball.rotation_around_x += input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_rotation_speed;
          state.arcball.rotation_around_y -= input.mouse_position_delta.logical.x as f32 * settings.arcball.mouse_movement_rotation_speed;
        }
        state.arcball.rotation_around_x = state.arcball.rotation_around_x.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
        state.arcball.rotation_around_y = state.arcball.rotation_around_y % (PI * 2.0);
        let rotor = Rotor3::from_euler_angles(0.0, state.arcball.rotation_around_x, state.arcball.rotation_around_y).normalized();

        // Panning (mouse)
        if input.secondary_mouse_button_down {
          if input.primary_mouse_button_down {
            // Y-up is negative, so multiply by -1.0.
            state.target += Vec3::unit_z().rotated_by(rotor) * input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_panning_speed * -1.0;
          } else {
            state.target += Vec3::unit_x().rotated_by(rotor) * input.mouse_position_delta.logical.x as f32 * settings.arcball.mouse_movement_panning_speed;
            // Y-up is negative, so multiply by -1.0.
            state.target += Vec3::unit_y().rotated_by(rotor) * input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_panning_speed * -1.0;
          }
        }

        // Panning (keyboard)
        let frame_delta = frame_duration.into_seconds() as f32;
        let keyboard_panning_speed = settings.arcball.keyboard_panning_speed * frame_delta;
        if input.forward_key_down {
          state.target += Vec3::unit_z().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.left_key_down {
          state.target += Vec3::unit_x().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }
        if input.backward_key_down {
          state.target += Vec3::unit_z().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }
        if input.right_key_down {
          state.target += Vec3::unit_x().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.up_key_down {
          state.target += Vec3::unit_y().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.down_key_down {
          state.target += Vec3::unit_y().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }

        self.target = state.target;

        // Camera position
        // Distance is positive, but moving backwards is negative-Z, so multiply by -1.0.
        self.position = state.target + Vec3::unit_z().rotated_by(rotor) * state.arcball.distance * -1.0
      }
    };
  }
}
