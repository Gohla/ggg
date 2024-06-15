use std::f32::consts::{FRAC_PI_2, PI};

use ultraviolet::{Mat4, Rotor3, Vec3};

use common::input::{KeyboardKey, MouseButton, RawInput};
use common::screen::{PhysicalSize, ScreenDelta};
use common::time::Offset;

#[cfg(feature = "debugging_gui")]
pub mod debug;
mod matrix;

// Camera settings

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraSettings {
  // View
  pub movement_type: MovementType,
  pub arcball: Arcball,
  pub fly: Fly,
  pub target: Vec3,
  // Projection
  pub projection_type: ProjectionType,
  pub perspective: Perspective,
  pub orthographic: Orthographic,
  pub near: f32,
  pub far: f32,
}

impl Default for CameraSettings {
  fn default() -> Self {
    Self {
      movement_type: MovementType::Arcball,
      arcball: Default::default(),
      fly: Default::default(),
      target: Vec3::zero(),

      projection_type: ProjectionType::Perspective,
      perspective: Default::default(),
      orthographic: Default::default(),
      near: 0.1,
      far: 1000.0,
    }
  }
}

impl CameraSettings {
  pub fn new(
    movement_type: MovementType,
    arcball: Arcball,
    fly: Fly,
    target: Vec3,
    projection_type: ProjectionType,
    perspective: Perspective,
    orthographic: Orthographic,
    near: f32,
    far: f32,
  ) -> Self {
    Self {
      movement_type,
      arcball,
      fly,
      target,

      projection_type,
      perspective,
      orthographic,
      near,
      far,

      ..Self::default()
    }
  }

  pub fn with_defaults(
    movement_type: MovementType,
    projection_type: ProjectionType,
  ) -> Self {
    Self {
      movement_type,
      projection_type,
      ..Self::default()
    }
  }

  pub fn with_defaults_arcball_perspective() -> Self {
    Self::with_defaults(MovementType::Arcball, ProjectionType::Perspective)
  }

  pub fn with_defaults_arcball_orthographic() -> Self {
    Self::with_defaults(MovementType::Arcball, ProjectionType::Orthographic)
  }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Arcball {
  pub mouse_movement_panning_speed: f32,
  pub keyboard_panning_speed: f32,
  pub debug_gui_panning_speed: f32,

  pub mouse_scroll_distance_speed: f32,
  pub debug_gui_distance_speed: f32,
  pub distance: f32,

  pub mouse_movement_rotation_speed: f32,
  pub debug_gui_rotation_speed: f32,
  pub rotation_around_x: f32,
  pub rotation_around_y: f32,
}

impl Default for Arcball {
  fn default() -> Self {
    Self {
      mouse_movement_panning_speed: 0.0025,
      keyboard_panning_speed: 1.0,
      debug_gui_panning_speed: 1.0,

      mouse_scroll_distance_speed: 0.1,
      debug_gui_distance_speed: 0.1,
      distance: 1.0,

      mouse_movement_rotation_speed: 0.0025,
      debug_gui_rotation_speed: 0.01,
      rotation_around_x: 0.0,
      rotation_around_y: 0.0,
    }
  }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Fly {}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MovementType {
  Arcball,
  Fly,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Perspective {
  pub vertical_fov_radians: f32,
}

impl Default for Perspective {
  fn default() -> Self {
    Self {
      vertical_fov_radians: 60.0f32.to_radians(),
    }
  }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Orthographic {}

impl Default for Orthographic {
  fn default() -> Self {
    Self {}
  }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ProjectionType {
  Perspective,
  Orthographic,
}


// Camera

#[derive(Copy, Clone, Debug)]
pub struct Camera {
  viewport: PhysicalSize,
  position: Vec3,
  direction: Vec3,
  direction_inverse: Vec3,
  view: Mat4,
  view_inverse: Mat4,
  projection: Mat4,
  projection_inverse: Mat4,
  view_projection: Mat4,
  view_projection_inverse: Mat4,
}

impl Camera {
  #[inline]
  pub fn new(viewport: PhysicalSize, settings: &mut CameraSettings) -> Self {
    let mut camera = Self {
      viewport,
      position: Vec3::zero(),
      direction: Vec3::one(),
      direction_inverse: Vec3::one() * -1.0,
      view: Mat4::identity(),
      view_inverse: Mat4::identity().inversed(),
      projection: Mat4::identity(),
      projection_inverse: Mat4::identity().inversed(),
      view_projection: Mat4::identity(),
      view_projection_inverse: Mat4::identity().inversed(),
    };
    camera.update(settings, &CameraInput::default(), Offset::default());
    camera
  }

  #[inline]
  pub fn set_viewport(&mut self, viewport: PhysicalSize) { self.viewport = viewport; }

  /// Gets the position of the camera (i.e., the eye of the camera).
  #[inline]
  pub fn get_position(&self) -> Vec3 { self.position }

  /// Gets the normalized direction of the camera (i.e., vector from position to target).
  #[inline]
  pub fn get_direction(&self) -> Vec3 { self.direction }

  /// Gets the normalized inverse direction of the camera (i.e., vector from target to position).
  #[inline]
  pub fn get_direction_inverse(&self) -> Vec3 { self.direction_inverse }

  /// Gets the view matrix.
  #[inline]
  pub fn get_view_matrix(&self) -> Mat4 { self.view }

  /// Gets the inversed view matrix.
  #[inline]
  pub fn get_view_inverse_matrix(&self) -> Mat4 { self.view_inverse }

  /// Gets the view-projection matrix.
  #[inline]
  pub fn get_view_projection_matrix(&self) -> Mat4 { self.view_projection }

  /// Gets the inversed view-projection matrix.
  #[inline]
  pub fn get_view_projection_inverse_matrix(&self) -> Mat4 { self.view_projection_inverse }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to view coordinates (in meters,
  /// relative to the center of the screen).
  #[inline]
  pub fn screen_to_view(&self, x: f32, y: f32) -> Vec3 {
    let (width, height): (f64, f64) = self.viewport.into();
    let x = 2.0 * x / width as f32 - 1.0;
    let y = 2.0 * y / height as f32 - 1.0;
    let vec = Vec3::new(x, y, 0.0);
    Vec3::from_homogeneous_point(self.view_projection_inverse * vec.into_homogeneous_point())
  }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to world coordinates (in meters,
  /// absolute).
  #[inline]
  pub fn screen_to_world(&self, x: f32, y: f32) -> Vec3 {
    self.position + self.screen_to_view(x, y)
  }


  pub fn update(
    &mut self,
    settings: &mut CameraSettings,
    input: &CameraInput,
    frame_delta: Offset,
  ) {
    let (width, height): (f64, f64) = self.viewport.into();
    let width = width as f32;
    let height = height as f32;

    self.position = match settings.movement_type {
      MovementType::Arcball => {
        // Rotation
        settings.arcball.distance += input.mouse_wheel_scroll_delta * settings.arcball.mouse_scroll_distance_speed * -1.0; // Scrolling up should zoom in, decreasing distance, so multiply by -1.0.
        if settings.arcball.distance < 0.1 { settings.arcball.distance = 0.1; }
        if input.primary_mouse_button_down && !input.secondary_mouse_button_down {
          settings.arcball.rotation_around_x += input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_rotation_speed;
          settings.arcball.rotation_around_y -= input.mouse_position_delta.logical.x as f32 * settings.arcball.mouse_movement_rotation_speed;
        }
        settings.arcball.rotation_around_x = settings.arcball.rotation_around_x.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
        settings.arcball.rotation_around_y = settings.arcball.rotation_around_y % (PI * 2.0);
        let rotor = Rotor3::from_euler_angles(0.0, settings.arcball.rotation_around_x, settings.arcball.rotation_around_y).normalized();
        // Panning (mouse)
        if input.secondary_mouse_button_down {
          if input.primary_mouse_button_down {
            settings.target += Vec3::unit_z().rotated_by(rotor) * input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_panning_speed * -1.0; // Y-up is negative, so multiply by -1.0.
          } else {
            settings.target += Vec3::unit_x().rotated_by(rotor) * input.mouse_position_delta.logical.x as f32 * settings.arcball.mouse_movement_panning_speed;
            settings.target += Vec3::unit_y().rotated_by(rotor) * input.mouse_position_delta.logical.y as f32 * settings.arcball.mouse_movement_panning_speed * -1.0; // Y-up is negative, so multiply by -1.0.
          }
        }
        // Panning (keyboard)
        let frame_delta = frame_delta.into_seconds() as f32;
        let keyboard_panning_speed = settings.arcball.keyboard_panning_speed * frame_delta;
        if input.forward_key_down {
          settings.target += Vec3::unit_z().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.left_key_down {
          settings.target += Vec3::unit_x().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }
        if input.backward_key_down {
          settings.target += Vec3::unit_z().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }
        if input.right_key_down {
          settings.target += Vec3::unit_x().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.up_key_down {
          settings.target += Vec3::unit_y().rotated_by(rotor) * keyboard_panning_speed;
        }
        if input.down_key_down {
          settings.target += Vec3::unit_y().rotated_by(rotor) * keyboard_panning_speed * -1.0;
        }
        // Camera position
        settings.target + Vec3::unit_z().rotated_by(rotor) * settings.arcball.distance * -1.0 // Distance is positive, but moving backwards is negative-Z, so multiply by -1.0.
      }
      MovementType::Fly => Vec3::zero(),
    };
    self.direction = (settings.target - self.position).normalized();
    self.direction_inverse = self.direction * -1.0;

    // View matrix.
    self.view = matrix::look_at_lh(self.position, settings.target, Vec3::unit_y());
    self.view_inverse = self.view.inversed();

    // Projection matrix
    let aspect_ratio = width / height;
    self.projection = match settings.projection_type {
      ProjectionType::Orthographic => {
        let zoom_factor = (settings.target - self.position).mag().abs();
        let width = aspect_ratio * zoom_factor;
        let height = zoom_factor;
        let left = -width / 2.0;
        let right = width / 2.0;
        let bottom = -height / 2.0;
        let top = height / 2.0;
        matrix::orthographic_reversed_lh_yup_wgpu_dx(left, right, bottom, top, settings.near, settings.far)
      }
      ProjectionType::Perspective => {
        matrix::perspective_infinite_reversed_lh_yup_wgpu_dx(settings.perspective.vertical_fov_radians, aspect_ratio, settings.near)
      }
    };
    self.projection_inverse = self.projection.inversed();

    self.view_projection = self.projection * self.view;
    self.view_projection_inverse = self.view_projection.inversed();
  }
}


// Input

#[derive(Default, Clone, Debug)]
pub struct CameraInput {
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

impl From<&RawInput> for CameraInput {
  fn from(input: &RawInput) -> Self {
    CameraInput {
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
