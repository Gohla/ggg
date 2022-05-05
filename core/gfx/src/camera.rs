use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};

use ultraviolet::{Mat4, Rotor3, Vec3, Vec4};

use common::input::{MouseButton, RawInput};
use common::screen::{PhysicalSize, ScreenDelta};
use common::timing::Duration;

// Camera

#[derive(Debug)]
pub struct Camera {
  // View
  pub viewport: PhysicalSize,
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
  // Internals
  position: Vec3,
  view: Mat4,
  view_inverse: Mat4,
  projection: Mat4,
  projection_inverse: Mat4,
  view_projection: Mat4,
  view_projection_inverse: Mat4,
}

impl Default for Camera {
  fn default() -> Self {
    Self {
      viewport: Default::default(),
      movement_type: MovementType::Arcball,
      arcball: Default::default(),
      fly: Default::default(),
      target: Vec3::zero(),

      projection_type: ProjectionType::Perspective,
      perspective: Default::default(),
      orthographic: Default::default(),
      near: 0.1,
      far: 1000.0,

      position: Vec3::zero(),
      view: Mat4::identity(),
      view_inverse: Mat4::identity().inversed(),
      projection: Mat4::identity(),
      projection_inverse: Mat4::identity().inversed(),
      view_projection: Mat4::identity(),
      view_projection_inverse: Mat4::identity().inversed(),
    }
  }
}

#[derive(Debug)]
pub struct Arcball {
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
      mouse_scroll_distance_speed: 5.0,
      debug_gui_distance_speed: 1.0,
      distance: -1.0,
      mouse_movement_rotation_speed: 0.5,
      debug_gui_rotation_speed: 0.01,
      rotation_around_x: 0.0,
      rotation_around_y: 0.0,
    }
  }
}

#[derive(Default, Debug)]
pub struct Fly {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum MovementType {
  Arcball,
  Fly,
}

#[derive(Debug)]
pub struct Perspective {
  pub vertical_fov_radians: f32,
}

impl Default for Perspective {
  fn default() -> Self {
    Self {
      vertical_fov_radians: 45.0f32.to_radians(),
    }
  }
}

#[derive(Debug)]
pub struct Orthographic {}

impl Default for Orthographic {
  fn default() -> Self {
    Self {}
  }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum ProjectionType {
  Perspective,
  Orthographic,
}

impl Camera {
  pub fn new(
    viewport: PhysicalSize,
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
      viewport,
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
    viewport: PhysicalSize,
    movement_type: MovementType,
    projection_type: ProjectionType,
  ) -> Camera {
    Self {
      viewport,
      movement_type,
      projection_type,
      ..Self::default()
    }
  }

  pub fn with_defaults_arcball_perspective(viewport: PhysicalSize) -> Self {
    Self::with_defaults(viewport, MovementType::Arcball, ProjectionType::Perspective)
  }

  pub fn with_defaults_arcball_orthographic(viewport: PhysicalSize) -> Self {
    Self::with_defaults(viewport, MovementType::Arcball, ProjectionType::Orthographic)
  }


  /// Gets the position of the camera (i.e., the eye of the camera).
  #[inline]
  pub fn get_position(&self) -> Vec3 { self.position }

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
    input: &CameraInput,
    frame_delta: Duration,
  ) {
    let (width, height): (f64, f64) = self.viewport.into();
    let width = width as f32;
    let height = height as f32;

    self.position = match self.movement_type {
      MovementType::Arcball => {
        let frame_delta = frame_delta.as_s() as f32;
        let distance_speed = self.arcball.mouse_scroll_distance_speed * frame_delta;
        self.arcball.distance += input.mouse_wheel_scroll_delta * distance_speed;
        if input.mouse_buttons.contains(&MouseButton::Left) {
          let rotation_speed = self.arcball.mouse_movement_rotation_speed * frame_delta;
          self.arcball.rotation_around_x += input.mouse_position_delta.logical.y as f32 * rotation_speed;
          self.arcball.rotation_around_y -= input.mouse_position_delta.logical.x as f32 * rotation_speed;
        }
        self.arcball.rotation_around_x = self.arcball.rotation_around_x.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
        self.arcball.rotation_around_y = self.arcball.rotation_around_y % (PI * 2.0);
        let mut position = {
          let mut position = self.target;
          position.z += self.arcball.distance;
          position
        };
        Rotor3::from_euler_angles(0.0, self.arcball.rotation_around_x, self.arcball.rotation_around_y).rotate_vec(&mut position);
        position
      }
      MovementType::Fly => Vec3::zero(),
    };

    // View matrix.
    self.view = look_at_lh(self.position, self.target, Vec3::unit_y());
    self.view_inverse = self.view.inversed();

    // Projection matrix
    let aspect_ratio = width / height;
    self.projection = match self.projection_type {
      ProjectionType::Orthographic => {
        let zoom_factor = (self.target - self.position).mag().abs();
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
    self.projection_inverse = self.projection.inversed();

    self.view_projection = self.projection * self.view;
    self.view_projection_inverse = self.view_projection.inversed();
  }
}

// Input

#[derive(Default, Clone, Debug)]
pub struct CameraInput {
  mouse_buttons: HashSet<MouseButton>,
  mouse_position_delta: ScreenDelta,
  mouse_wheel_scroll_delta: f32,
}

impl From<&RawInput> for CameraInput {
  fn from(input: &RawInput) -> Self {
    CameraInput {
      mouse_buttons: input.mouse_buttons.clone(),
      mouse_position_delta: input.mouse_position_delta,
      mouse_wheel_scroll_delta: input.mouse_wheel_pixel_delta.physical.y as f32 + input.mouse_wheel_line_delta.vertical as f32,
    }
  }
}

// Debugging

#[cfg(feature = "debugging_gui")]
#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraDebugging {
  pub show_window: bool,
  pub window_anchor: Option<egui::Align2>,
}

#[cfg(feature = "debugging_gui")]
impl CameraDebugging {
  pub fn show_debugging_gui_window(&mut self, ctx: &egui::Context, camera: &mut Camera) {
    if !self.show_window { return; }
    let mut window = egui::Window::new("Camera");
    if let Some(anchor) = self.window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window
      .open(&mut self.show_window)
      .auto_sized()
      .show(ctx, |ui| camera.draw_debugging_gui(ui, &mut self.window_anchor));
  }

  pub fn add_to_menu(&mut self, ui: &mut egui::Ui) {
    ui.checkbox(&mut self.show_window, "Camera");
  }
}

#[cfg(feature = "debugging_gui")]
impl Camera {
  pub fn draw_debugging_gui(&mut self, ui: &mut egui::Ui, window_anchor: &mut Option<egui::Align2>) {
    use egui::ComboBox;
    use gui_widget::*;
    ui.horizontal(|ui| {
      ui.label("Anchor");
      ui.select_align2(window_anchor);
    });
    ui.collapsing_open_with_grid("View", "Grid", |ui| {
      ui.label("Movement mode");
      ComboBox::from_id_source("Movement mode")
        .selected_text(format!("{:?}", self.movement_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.movement_type, MovementType::Arcball, "Arcball");
          ui.selectable_value(&mut self.movement_type, MovementType::Fly, "Fly");
        });
      ui.end_row();
      match self.movement_type {
        MovementType::Arcball => {
          ui.label("Distance");
          ui.horizontal(|ui| {
            ui.drag_unlabelled(&mut self.arcball.distance, self.arcball.debug_gui_distance_speed);
            ui.drag_range("mouse: ", &mut self.arcball.mouse_scroll_distance_speed, 1.0, 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut self.arcball.debug_gui_distance_speed, 1.0, 0.0..=f32::INFINITY);
          });
          ui.end_row();
          ui.label("Rotation");
          ui.horizontal(|ui| {
            ui.drag("x: ", &mut self.arcball.rotation_around_x, 0.01);
            ui.drag("y: ", &mut self.arcball.rotation_around_y, 0.01);
            ui.drag_range("mouse: ", &mut self.arcball.mouse_movement_rotation_speed, 1., 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut self.arcball.debug_gui_rotation_speed, 1.0, 0.0..=f32::INFINITY);
          });
          ui.end_row();
        }
        MovementType::Fly => {}
      }
      ui.label("Position");
      ui.show_vec3(&self.position);
      ui.end_row();
      ui.label("Target");
      ui.drag_vec3(0.1, &mut self.target);
      ui.end_row();
    });
    ui.collapsing_open_with_grid("Projection", "Grid", |ui| {
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
          ui.label("Vertical FOV");
          ui.drag_angle(&mut self.perspective.vertical_fov_radians);
          ui.end_row();
        }
        ProjectionType::Orthographic => {}
      }
      ui.label("Viewport");
      ui.monospace(format!("{:.2}x{:.2} ({:.2})", self.viewport.width, self.viewport.height, self.viewport.ratio()));
      ui.end_row();
      ui.label("Frustum");
      ui.horizontal(|ui| {
        ui.drag("near: ", &mut self.near, 0.001);
        ui.drag("far: ", &mut self.far, 1.0);
      });
      ui.end_row();
    });
    ui.collapsing_with_grid("Matrices", "Grid", |ui| {
      ui.label("View matrix");
      ui.show_mat4(&self.view);
      ui.end_row();
      ui.label("Projection matrix");
      ui.show_mat4(&self.projection);
      ui.end_row();
      ui.label("View-projection matrix");
      ui.show_mat4(&self.view_projection);
      ui.end_row();
      ui.label("View-projection matrix inverse");
      ui.show_mat4(&self.view_projection_inverse);
      ui.end_row();
    });
  }
}

// Internals

#[inline]
fn look_at_lh(
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
fn perspective_lh_yup_wgpu_dx(
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
  Mat4::new( // @formatter:off
    Vec4::new(2.0 / rml, 0.0      , 0.0       , 0.0),
    Vec4::new(0.0      , 2.0 / tmb, 0.0       , 0.0),
    Vec4::new(0.0      , 0.0      , 1.0 / fmn , 0.0),
    Vec4::new(lpr / lmr, tpb / bmt, near / fmn, 1.0),
  ) // @formatter:on
}
