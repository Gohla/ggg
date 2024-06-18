use ultraviolet::{Mat4, Vec3};

use common::screen::PhysicalSize;
use common::time::Offset;

use crate::camera::controller::{CameraController, CameraControllerData, CameraControllerInput, CameraControllerSettings};
use crate::camera::projection::{CameraProjection, CameraProjectionSettings};

pub mod controller;
pub mod projection;
#[cfg(feature = "inspector_gui")]
pub mod inspector;

/// Camera settings
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraSettings {
  pub controller: CameraControllerSettings,
  pub projection: CameraProjectionSettings,
}

impl CameraSettings {
  #[inline]
  pub fn perspective(mut self) -> Self {
    self.projection = self.projection.perspective();
    self
  }
  #[inline]
  pub fn orthographic(mut self) -> Self {
    self.projection = self.projection.orthographic();
    self
  }
}

/// Camera data
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraData {
  pub controller: CameraControllerData,
}

/// Camera
#[derive(Copy, Clone, Debug)]
pub struct Camera {
  pub controller: CameraController,
  pub projection: CameraProjection,
}

impl Camera {
  #[inline]
  pub fn new(
    data: &mut CameraData,
    settings: &CameraSettings,
    viewport: PhysicalSize,
  ) -> Self {
    let mut controller = CameraController::default();
    controller.update(&mut data.controller, &settings.controller, &CameraControllerInput::default(), Offset::zero());
    let mut projection = CameraProjection::new(viewport);
    projection.update(&settings.projection, controller.position(), controller.target());
    Self { controller, projection }
  }


  /// Gets the position of this camera. That is, the eye of the camera.
  #[inline]
  pub fn position(&self) -> Vec3 { self.controller.position() }
  // /// Gets the target of this camera. That is, the target the eyes of the camera are looking at.
  // #[inline]
  // pub fn target(&self) -> Vec3 { self.controller.target() }

  /// Gets the viewport.
  #[inline]
  pub fn viewport(&self) -> &PhysicalSize { self.projection.viewport() }

  /// Gets the view matrix.
  #[inline]
  pub fn direction_vector(&self) -> Vec3 { self.projection.direction_vector() }
  /// Gets the inverse view matrix.
  #[inline]
  pub fn inverse_direction_vector(&self) -> Vec3 { self.projection.inverse_direction_vector() }

  /// Gets the view matrix.
  #[inline]
  pub fn view_matrix(&self) -> &Mat4 { self.projection.view_matrix() }
  /// Gets the inverse view matrix.
  #[inline]
  pub fn inverse_view_matrix(&self) -> &Mat4 { self.projection.inverse_view_matrix() }
  /// Gets the projection matrix.
  #[inline]
  pub fn projection_matrix(&self) -> &Mat4 { self.projection.projection_matrix() }
  /// Gets the inverse projection matrix.
  #[inline]
  pub fn inverse_projection_matrix(&self) -> &Mat4 { self.projection.inverse_projection_matrix() }
  /// Gets the view-projection matrix.
  #[inline]
  pub fn view_projection_matrix(&self) -> &Mat4 { self.projection.view_projection_matrix() }
  /// Gets the inverse view-projection matrix.
  #[inline]
  pub fn inverse_view_projection_matrix(&self) -> &Mat4 { self.projection.inverse_view_projection_matrix() }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to view coordinates (in meters,
  /// relative to the center of the screen).
  #[inline]
  pub fn screen_to_view(&self, x: f32, y: f32) -> Vec3 { self.projection.screen_to_view(x, y) }
  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to world coordinates (in meters,
  /// absolute).
  #[inline]
  pub fn screen_to_world(&self, x: f32, y: f32) -> Vec3 {
    self.controller.position() + self.projection.screen_to_view(x, y)
  }


  /// Sets the `viewport`.
  #[inline]
  pub fn set_viewport(&mut self, viewport: PhysicalSize) {
    self.projection.set_viewport(viewport);
  }

  /// Update the camera controller and projection.
  #[inline]
  pub fn update(
    &mut self,
    data: &mut CameraData,
    settings: &CameraSettings,
    input: &CameraControllerInput,
    frame_duration: Offset,
  ) {
    self.controller.update(&mut data.controller, &settings.controller, input, frame_duration);
    self.projection.update(&settings.projection, self.controller.position(), self.controller.target());
  }
}
