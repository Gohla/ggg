use std::ops::{Deref, DerefMut};

use common::screen::PhysicalSize;
use common::time::Offset;

use crate::camera::{Camera, CameraSettings, CameraState};
use crate::camera::controller::CameraControllerInput;

/// Camera system that manages multiple camera's of which one is active.
#[derive(Clone, Debug)]
pub struct CameraSystem {
  data: Vec<CameraData>,
  active_camera: usize,

  default_data: CameraData,
  cameras: Vec<Camera>,
}

/// Data associated with a camera, consisting of [state](CameraState) and [settings](CameraSettings).
#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraData {
  pub state: CameraState,
  pub settings: CameraSettings,
}
impl CameraData {
  #[inline]
  pub fn create_camera(&mut self, viewport: PhysicalSize) -> Camera {
    Camera::new(&mut self.state, &self.settings, viewport)
  }
}

/// State of a camera system. Use it to create the camera system. Turn a camera system back into state with
/// `camera_system.into()`.
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraSystemState {
  data: Vec<CameraData>,
  active_camera: usize,
}
impl CameraSystemState {
  /// Take `self` and use it to create a camera system, leaving `CameraSystemState::default()` in place of `self`.
  pub fn take_into(&mut self, default_data: CameraData, viewport: PhysicalSize) -> CameraSystem {
    let state = std::mem::take(self);
    CameraSystem::new(state, default_data, viewport)
  }
}

impl CameraSystem {
  /// Creates a new camera system from `state`, using `default_data` to create new camera's, and using `viewport` to
  /// initialize the camera's in this function.
  pub fn new(
    mut state: CameraSystemState,
    default_data: CameraData,
    viewport: PhysicalSize,
  ) -> Self {
    let cameras = state.data.iter_mut()
      .map(|data| data.create_camera(viewport))
      .collect();
    let mut system = Self {
      data: state.data,
      active_camera: state.active_camera,
      cameras,
      default_data,
    };

    // Ensure there are at least as many cameras as the `active_camera` index could select.
    system.ensure_minimum_camera_count(state.active_camera + 1, viewport);

    system
  }

  /// Returns the default camera data.
  #[inline]
  pub fn default_data(&self) -> CameraData { self.default_data }

  /// Create camera's until there are `minimum_camera_count` camera's, using `viewport` to initialize new camera's.
  pub fn ensure_minimum_camera_count(&mut self, minimum_camera_count: usize, viewport: PhysicalSize) {
    let num_to_create_cameras = minimum_camera_count.saturating_sub(self.camera_count());
    for _ in 0..num_to_create_cameras {
      let mut data = self.default_data.clone();
      let camera = data.create_camera(viewport);
      self.data.push(data);
      self.cameras.push(camera);
    }
  }

  /// Sets the viewport for all camera's.
  pub fn set_viewport(&mut self, viewport: PhysicalSize) {
    for camera in &mut self.cameras {
      camera.set_viewport(viewport);
    }
  }
}
impl From<CameraSystem> for CameraSystemState {
  #[inline]
  fn from(manager: CameraSystem) -> Self {
    Self {
      data: manager.data,
      active_camera: manager.active_camera,
    }
  }
}

/// Combination of camera [state](CameraState), [settings](CameraSettings), and [runtime instance](Camera).
pub struct CombinedCamera<'c> {
  pub state: &'c mut CameraState,
  pub settings: &'c mut CameraSettings,
  pub camera: &'c mut Camera,
}
impl<'c> CombinedCamera<'c> {
  pub fn from(data: &'c mut CameraData, camera: &'c mut Camera) -> Self {
    Self { state: &mut data.state, settings: &mut data.settings, camera }
  }
}
impl Deref for CombinedCamera<'_> {
  type Target = Camera;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.camera }
}
impl DerefMut for CombinedCamera<'_> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target { self.camera }
}
impl<'c> CombinedCamera<'c> {
  /// Update the camera of this combined camera.
  #[inline]
  pub fn update(&mut self, input: &CameraControllerInput, frame_duration: Offset) {
    self.camera.update(self.state, self.settings, input, frame_duration);
  }
}

impl CameraSystem {
  /// Gets the number of camera's in the system.
  #[inline]
  pub fn camera_count(&self) -> usize {
    self.data.len()
  }
  /// Iterative over the camera's in the system.
  #[inline]
  pub fn cameras(&mut self) -> impl Iterator<Item=CombinedCamera> {
    self.data.iter_mut()
      .zip(self.cameras.iter_mut())
      .map(|(data, camera)| CombinedCamera::from(data, camera))
  }

  /// Gets the camera at `index`.
  ///
  /// # Panics
  ///
  /// Panics if there is no camera at that index.
  #[inline]
  pub fn camera_at(&mut self, index: usize) -> CombinedCamera {
    let data = &mut self.data[index];
    let camera = &mut self.cameras[index];
    CombinedCamera::from(data, camera)
  }

  /// Gets the active camera.
  ///
  /// # Panics
  ///
  /// Panics if there is no camera at the active camera index.
  #[inline]
  pub fn active_camera(&mut self) -> CombinedCamera {
    self.camera_at(self.active_camera)
  }

  /// Gets a mutable reference to the active camera index.
  #[inline]
  pub fn active_camera_index_mut(&mut self) -> &mut usize {
    &mut self.active_camera
  }
}
