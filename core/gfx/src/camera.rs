use egui::{ComboBox, CtxRef};
use ultraviolet::{Mat4, Vec3, Vec4};

use common::input::{KeyboardButton, RawInput};
use common::screen::PhysicalSize;
use common::timing::Duration;
use gui_widget::*;

#[derive(Debug)]
pub struct Camera {
  pub position: Vec3,
  pub panning_speed: f32,

  pub viewport: PhysicalSize,
  pub projection_type: ProjectionType,
  pub perspective: PerspectiveProjection,
  pub orthographic: OrthographicProjection,
  pub near: f32,
  pub far: f32,

  pub show_debug_gui: bool,

  view_projection: Mat4,
  view_projection_inverse: Mat4,
}

#[derive(Debug)]
pub struct PerspectiveProjection {
  pub vertical_fov_radians: f32,
  pub fov_change_speed: f32,
}

impl Default for PerspectiveProjection {
  fn default() -> Self {
    Self {
      vertical_fov_radians: 45.0f32.to_radians(),
      fov_change_speed: 0.1,
    }
  }
}

#[derive(Debug)]
pub struct OrthographicProjection {
  pub zoom: f32,
  pub zoom_change_speed: f32,
}

impl Default for OrthographicProjection {
  fn default() -> Self {
    Self {
      zoom: 1.0,
      zoom_change_speed: 0.1,
    }
  }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum ProjectionType {
  Perspective,
  Orthographic,
}

impl Camera {
  pub fn new(
    position: Vec3,
    panning_speed: f32,
    viewport: PhysicalSize,
    projection_type: ProjectionType,
    perspective_projection: PerspectiveProjection,
    orthographic_projection: OrthographicProjection,
    near: f32,
    far: f32,
  ) -> Camera {
    Camera {
      position,
      panning_speed,

      viewport,
      projection_type,
      perspective: perspective_projection,
      orthographic: orthographic_projection,
      near,
      far,

      show_debug_gui: false,

      view_projection: Mat4::identity(),
      view_projection_inverse: Mat4::identity().inversed(),
    }
  }

  pub fn with_defaults(
    viewport: PhysicalSize,
    projection_type: ProjectionType,
  ) -> Camera {
    Self::new(
      Vec3::new(0.0, 0.0, -1.0),
      10.0,
      viewport,
      projection_type,
      PerspectiveProjection::default(),
      OrthographicProjection::default(),
      0.1,
      5000.0,
    )
  }

  pub fn with_defaults_perspective(viewport: PhysicalSize) -> Self {
    Self::with_defaults(viewport, ProjectionType::Perspective)
  }

  pub fn with_defaults_orthographic(viewport: PhysicalSize) -> Self {
    Self::with_defaults(viewport, ProjectionType::Orthographic)
  }


  /// Gets the view-projection matrix.
  #[inline]
  pub fn get_view_projection_matrix(&self) -> Mat4 { self.view_projection }

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
    input: &CameraInput,
    frame_delta: Duration,
    gui_context: &CtxRef,
  ) {
    let panning_speed = self.panning_speed * frame_delta.as_s() as f32;
    if input.move_up { self.position.y += panning_speed };
    if input.move_right { self.position.x += panning_speed };
    if input.move_down { self.position.y -= panning_speed };
    if input.move_left { self.position.x -= panning_speed };
    match self.projection_type {
      ProjectionType::Perspective => {
        self.position.z += input.zoom_delta;
      }
      ProjectionType::Orthographic => {
        self.orthographic.zoom *= 1.0 - input.zoom_delta * self.orthographic.zoom_change_speed;
      }
    }

    let (width, height): (f64, f64) = self.viewport.into();
    let width = width as f32;
    let height = height as f32;

    // View matrix.
    let eye = Vec3::new(self.position.x, self.position.y, self.position.z);
    let at = Vec3::new(self.position.x, self.position.y, self.position.z + 1.0);
    let up = Vec3::unit_y();
    let view = look_at_lh(eye, at, up);

    // Projection matrix
    let aspect_ratio = width / height;
    let projection = match self.projection_type {
      ProjectionType::Orthographic => {
        let zoom_factor = self.orthographic.zoom;
        let width = aspect_ratio * zoom_factor;
        let height = zoom_factor;
        let left = -width / 2.0;
        let right = width / 2.0;
        let bottom = -height / 2.0;
        let top = height / 2.0;
        orthographic_lh_yup_wgpu_dx(left, right, bottom, top, self.near, self.far)
      }
      ProjectionType::Perspective => {
        perspective_lh_yup_wgpu_dx(self.perspective.vertical_fov_radians, aspect_ratio, self.near, self.far)
      }
    };

    let view_projection = projection * view;
    self.view_projection = view_projection;
    self.view_projection_inverse = view_projection.inversed();

    if !self.show_debug_gui { return; }

    gui_context.window("Debug Camera", |ui| {
      ui.collapsing_open_with_grid("View", "Grid", |ui| {
        ui.label("Movement");
        ui.horizontal(|ui| {
          if input.move_up { ui.label("Up"); }
          if input.move_right { ui.label("Right"); }
          if input.move_down { ui.label("Down"); }
          if input.move_left { ui.label("Left"); }
        });
        ui.end_row();
        ui.label("Panning Speed");
        ui.horizontal(|ui| {
          ui.drag_unlabelled(&mut self.panning_speed, 0.1);
          ui.show_f32_2(panning_speed);
        });
        ui.end_row();
        ui.label("Position");
        ui.drag_vec3(0.1, &mut self.position);
        ui.end_row();
        ui.label("Eye");
        ui.show_vec3(&eye);
        ui.end_row();
        ui.label("At");
        ui.show_vec3(&at);
        ui.end_row();
        ui.label("Up");
        ui.show_vec3(&up);
        ui.end_row();
      });
      ui.collapsing_open_with_grid("Projection", "Grid", |ui| {
        ui.label("Viewport");
        ui.horizontal(|ui| {
          ui.monospace(format!("width: {:.2}", self.viewport.width));
          ui.monospace(format!("height: {:.2}", self.viewport.height));
          ui.monospace(format!("aspect ratio: {:.2}", aspect_ratio));
        });
        ui.end_row();
        ui.label("Projection mode");
        ComboBox::from_id_source("Projection mode")
          .selected_text(format!("{:?}", self.projection_type))
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.projection_type, ProjectionType::Perspective, "Perspective");
            ui.selectable_value(&mut self.projection_type, ProjectionType::Orthographic, "Orthographic");
          });
        ui.end_row();
        match self.projection_type {
          ProjectionType::Perspective => {
            ui.label("Perspective");
            ui.horizontal(|ui| {
              ui.horizontal(|ui| {
                ui.label("Vertical FOV");
                ui.drag_angle(&mut self.perspective.vertical_fov_radians);
                ui.drag("change speed: ", &mut self.perspective.fov_change_speed, 0.01);
              });
            });
            ui.end_row();
          }
          ProjectionType::Orthographic => {
            ui.label("Orthographic");
            ui.horizontal(|ui| {
              ui.label("Zoom");
              ui.drag_unlabelled(&mut self.orthographic.zoom, 0.1);
              ui.drag("change speed: ", &mut self.orthographic.zoom_change_speed, 0.01);
            });
            ui.end_row();
          }
        }
        ui.label("Near/Far");
        ui.horizontal(|ui| {
          ui.drag("near: ", &mut self.near, 0.001);
          ui.drag("far: ", &mut self.far, 1.0);
        });
        ui.end_row();
      });
      ui.collapsing_with_grid("Matrices", "Grid", |ui| {
        ui.label("View matrix");
        ui.show_mat4(&view);
        ui.end_row();
        ui.label("Projection matrix");
        ui.show_mat4(&projection);
        ui.end_row();
        ui.label("View-projection matrix");
        ui.show_mat4(&view_projection);
        ui.end_row();
        ui.label("View-projection matrix inverse");
        ui.show_mat4(&self.view_projection_inverse);
        ui.end_row();
      });
    });
  }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct CameraInput {
  pub move_up: bool,
  pub move_right: bool,
  pub move_down: bool,
  pub move_left: bool,
  pub zoom_delta: f32,
}

impl From<&RawInput> for CameraInput {
  fn from(input: &RawInput) -> Self {
    CameraInput {
      move_up: input.is_keyboard_button_down(KeyboardButton::W),
      move_right: input.is_keyboard_button_down(KeyboardButton::D),
      move_down: input.is_keyboard_button_down(KeyboardButton::S),
      move_left: input.is_keyboard_button_down(KeyboardButton::A),
      zoom_delta: input.mouse_wheel_pixel_delta.physical.y as f32 + input.mouse_wheel_line_delta.vertical as f32,
    }
  }
}

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

#[inline]
pub fn perspective_lh_yup_wgpu_dx(
  vertical_fov: f32,
  aspect_ratio: f32,
  near: f32,
  far: f32,
) -> Mat4 {
  // From: https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dxmatrixperspectivefovlh
  let t = (vertical_fov / 2.0).tan();
  let sy = 1.0 / t;
  let sx = sy / aspect_ratio;
  let fmn = far - near;
  Mat4::new( // @formatter:off
    Vec4::new(sx , 0.0, 0.0              , 0.0),
    Vec4::new(0.0, sy , 0.0              , 0.0),
    Vec4::new(0.0, 0.0, far / fmn        , 1.0),
    Vec4::new(0.0, 0.0, -near * far / fmn, 0.0),
  ) // @formatter:on
}

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
  Mat4::new( // @formatter:off
    Vec4::new(2.0 / rml, 0.0      , 0.0       , 0.0),
    Vec4::new(0.0      , 2.0 / tmb, 0.0       , 0.0),
    Vec4::new(0.0      , 0.0      , 1.0 / fmn , 0.0),
    Vec4::new(lpr / lmr, tpb / bmt, near / fmn, 1.0),
  ) // @formatter:on
}
