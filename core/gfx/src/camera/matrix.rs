use ultraviolet::{Mat4, Vec3, Vec4};

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
pub fn perspective_lh_yup_wgpu_dx(
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
pub fn perspective_infinite_lh_yup_wgpu_dx(
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
pub fn perspective_infinite_reversed_lh_yup_wgpu_dx(
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
pub fn orthographic_lh_yup_wgpu_dx(
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
pub fn orthographic_reversed_lh_yup_wgpu_dx(
  left: f32, right: f32,
  bottom: f32, top: f32,
  near: f32, far: f32,
) -> Mat4 {
  orthographic_lh_yup_wgpu_dx(left, right, bottom, top, far, near) // Note: far and near are swapped.
}
