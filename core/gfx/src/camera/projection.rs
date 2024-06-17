use ultraviolet::{Mat4, Vec3, Vec4};

use common::screen::PhysicalSize;

/// Camera projection settings.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraProjectionSettings {
  pub projection_type: ProjectionType,
  pub perspective: PerspectiveSettings,
  pub orthographic: OrthographicSettings,
  pub near: f32,
  pub far: f32,
}
impl Default for CameraProjectionSettings {
  #[inline]
  fn default() -> Self {
    Self {
      projection_type: ProjectionType::default(),
      perspective: PerspectiveSettings::default(),
      orthographic: OrthographicSettings::default(),
      near: 0.1,
      far: 1000.0,
    }
  }
}
impl CameraProjectionSettings {
  #[inline]
  pub fn perspective(mut self) -> Self {
    self.projection_type = ProjectionType::Perspective;
    self
  }
  #[inline]
  pub fn orthographic(mut self) -> Self {
    self.projection_type = ProjectionType::Orthographic;
    self
  }
}

/// Camera projection types.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ProjectionType {
  #[default]
  Perspective,
  Orthographic,
}

/// Perspective projection settings.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PerspectiveSettings {
  pub vertical_fov_radians: f32,
}
impl Default for PerspectiveSettings {
  #[inline]
  fn default() -> Self {
    Self {
      vertical_fov_radians: 60.0f32.to_radians(),
    }
  }
}

/// Orthographic projection settings.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OrthographicSettings {}


/// Camera projection.
#[derive(Default, Copy, Clone, Debug)]
pub struct CameraProjection {
  viewport: PhysicalSize,
  direction: Vec3,
  inverse_direction: Vec3,
  view: Mat4,
  inverse_view: Mat4,
  projection: Mat4,
  inverse_projection: Mat4,
  view_projection: Mat4,
  inverse_view_projection: Mat4,
}

impl CameraProjection {
  #[inline]
  pub fn new(viewport: PhysicalSize) -> Self {
    Self {
      viewport,
      direction: Vec3::one(),
      inverse_direction: Vec3::one() * -1.0,
      view: Mat4::identity(),
      inverse_view: Mat4::identity().inversed(),
      projection: Mat4::identity(),
      inverse_projection: Mat4::identity().inversed(),
      view_projection: Mat4::identity(),
      inverse_view_projection: Mat4::identity().inversed(),
    }
  }

  /// Gets the viewport.
  #[inline]
  pub fn viewport(&self) -> &PhysicalSize { &self.viewport }

  /// Gets the view matrix.
  #[inline]
  pub fn direction_vector(&self) -> Vec3 { self.direction }
  /// Gets the inverse view matrix.
  #[inline]
  pub fn inverse_direction_vector(&self) -> Vec3 { self.inverse_direction }

  /// Gets the view matrix.
  #[inline]
  pub fn view_matrix(&self) -> &Mat4 { &self.view }
  /// Gets the inverse view matrix.
  #[inline]
  pub fn inverse_view_matrix(&self) -> &Mat4 { &self.inverse_view }
  /// Gets the projection matrix.
  #[inline]
  pub fn projection_matrix(&self) -> &Mat4 { &self.projection }
  /// Gets the inverse projection matrix.
  #[inline]
  pub fn inverse_projection_matrix(&self) -> &Mat4 { &self.inverse_projection }
  /// Gets the view-projection matrix.
  #[inline]
  pub fn view_projection_matrix(&self) -> &Mat4 { &self.view_projection }
  /// Gets the inverse view-projection matrix.
  #[inline]
  pub fn inverse_view_projection_matrix(&self) -> &Mat4 { &self.inverse_view_projection }

  /// Converts screen coordinates (in pixels, relative to the top-left of the screen) to view coordinates (in meters,
  /// relative to the center of the screen).
  #[inline]
  pub fn screen_to_view(&self, x: f32, y: f32) -> Vec3 {
    let (width, height): (f64, f64) = self.viewport.into();
    let x = 2.0 * x / width as f32 - 1.0;
    let y = 2.0 * y / height as f32 - 1.0;
    let vec = Vec3::new(x, y, 0.0);
    Vec3::from_homogeneous_point(self.inverse_view_projection * vec.into_homogeneous_point())
  }

  /// Sets the `viewport`.
  #[inline]
  pub fn set_viewport(&mut self, viewport: PhysicalSize) {
    self.viewport = viewport;
  }
  /// Update the direction vectors and projection matrices of this camera projection.
  pub fn update(
    &mut self,
    settings: &CameraProjectionSettings,
    position: Vec3,
    target: Vec3,
  ) {
    let (width, height): (f64, f64) = self.viewport.into();
    let width = width as f32;
    let height = height as f32;

    self.direction = (target - position).normalized();
    self.inverse_direction = self.direction * -1.0;

    self.view = look_at_lh(position, target, Vec3::unit_y());
    self.inverse_view = self.view.inversed();

    let aspect_ratio = width / height;
    self.projection = match settings.projection_type {
      ProjectionType::Orthographic => {
        let zoom_factor = (target - position).mag().abs();
        let width = aspect_ratio * zoom_factor;
        let height = zoom_factor;
        let left = -width / 2.0;
        let right = width / 2.0;
        let bottom = -height / 2.0;
        let top = height / 2.0;
        orthographic_reversed_lh_yup_wgpu_dx(left, right, bottom, top, settings.near, settings.far)
      }
      ProjectionType::Perspective => {
        perspective_infinite_reversed_lh_yup_wgpu_dx(settings.perspective.vertical_fov_radians, aspect_ratio, settings.near)
      }
    };
    self.inverse_projection = self.projection.inversed();

    self.view_projection = self.projection * self.view;
    self.inverse_view_projection = self.view_projection.inversed();
  }
}


// Matrices

#[inline]
pub fn look_at_lh(
  position: Vec3,
  target: Vec3,
  up: Vec3,
) -> Mat4 {
  // From: https://docs.microsoft.com/en-us/previous-versions/windows/desktop/bb281710(v=vs.85)
  let z_axis = (target - position).normalized();
  let x_axis = up.cross(z_axis).normalized();
  let y_axis = z_axis.cross(x_axis);
  Mat4::new( // @formatter:off
             Vec4::new(x_axis.x                   , y_axis.x                   , z_axis.x                   , 0.0),
             Vec4::new(x_axis.y                   , y_axis.y                   , z_axis.y                   , 0.0),
             Vec4::new(x_axis.z                   , y_axis.z                   , z_axis.z                   , 0.0),
             Vec4::new(-x_axis.dot(position), -y_axis.dot(position), -z_axis.dot(position), 1.0),
  ) // @formatter:on
}

/// Creates a left-handed perspective projection matrix with 0-1 depth range.
#[allow(dead_code)]
#[inline]
fn perspective_lh_yup_wgpu_dx(
  vertical_fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
) -> Mat4 {
  // From: https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dxmatrixperspectivefovlh
  // From: https://github.com/bitshifter/glam-rs/blob/main/src/core/traits/projection.rs#L26
  let (sin_fov, cos_fov) = (0.5 * vertical_fov).sin_cos();
  let h = cos_fov / sin_fov;
  let w = h / aspect_ratio;
  let r = far / (far - near);
  Mat4::new( // @formatter:off
             Vec4::new(w  , 0.0, 0.0      , 0.0),
             Vec4::new(0.0, h  , 0.0      , 0.0),
             Vec4::new(0.0, 0.0, r        , 1.0),
             Vec4::new(0.0, 0.0, -r * near, 0.0),
  ) // @formatter:on
}

/// Creates an infinite left-handed perspective projection matrix with 0-1 depth range.
#[allow(dead_code)]
#[inline]
fn perspective_infinite_lh_yup_wgpu_dx(
  vertical_fov: f32,
  aspect_ratio: f32,
  near: f32,
) -> Mat4 {
  // From: https://github.com/bitshifter/glam-rs/blob/main/src/core/traits/projection.rs#L56
  let (sin_fov, cos_fov) = (0.5 * vertical_fov).sin_cos();
  let h = cos_fov / sin_fov;
  let w = h / aspect_ratio;
  Mat4::new( // @formatter:off
             Vec4::new(w  , 0.0, 0.0  , 0.0),
             Vec4::new(0.0, h  , 0.0  , 0.0),
             Vec4::new(0.0, 0.0, 1.0  , 1.0),
             Vec4::new(0.0, 0.0, -near, 0.0),
  ) // @formatter:on
}

/// Creates an infinite left-handed perspective projection matrix with 1-0 depth range.
#[inline]
fn perspective_infinite_reversed_lh_yup_wgpu_dx(
  vertical_fov: f32,
  aspect_ratio: f32,
  near: f32,
) -> Mat4 {
  // From: https://github.com/bitshifter/glam-rs/blob/main/src/core/traits/projection.rs#L70
  let (sin_fov, cos_fov) = (0.5 * vertical_fov).sin_cos();
  let h = cos_fov / sin_fov;
  let w = h / aspect_ratio;
  Mat4::new( // @formatter:off
             Vec4::new(w  , 0.0, 0.0 , 0.0),
             Vec4::new(0.0, h  , 0.0 , 0.0),
             Vec4::new(0.0, 0.0, 0.0 , 1.0),
             Vec4::new(0.0, 0.0, near, 0.0),
  ) // @formatter:on
}

/// Creates a left-handed orthographic projection matrix with 0-1 depth range.
#[allow(dead_code)]
#[inline]
fn orthographic_lh_yup_wgpu_dx(
  left: f32, right: f32,
  bottom: f32, top: f32,
  near: f32, far: f32,
) -> Mat4 {
  // From: https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dxmatrixorthooffcenterlh
  let rml = right - left;
  let lmr = left - right;
  let lpr = left + right;
  let tmb = top - bottom;
  let bmt = bottom - top;
  let tpb = top + bottom;
  let fmn = far - near;
  let nmf = near - far;
  Mat4::new( // @formatter:off
             Vec4::new(2.0 / rml, 0.0      , 0.0       , 0.0),
             Vec4::new(0.0      , 2.0 / tmb, 0.0       , 0.0),
             Vec4::new(0.0      , 0.0      , 1.0 / fmn , 0.0),
             Vec4::new(lpr / lmr, tpb / bmt, near / nmf, 1.0),
  ) // @formatter:on
}

/// Creates a left-handed orthographic projection matrix with 1-0 depth range.
#[inline]
fn orthographic_reversed_lh_yup_wgpu_dx(
  left: f32, right: f32,
  bottom: f32, top: f32,
  near: f32, far: f32,
) -> Mat4 {
  orthographic_lh_yup_wgpu_dx(left, right, bottom, top, far, near) // Note: far and near are swapped.
}
