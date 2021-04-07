use egui::{CollapsingHeader, CtxRef, Grid};
use ultraviolet::{Mat4, Vec2, Vec3};
use ultraviolet::projection;

use common::input::{KeyboardButton, RawInput};
use common::screen::PhysicalSize;
use common::timing::Duration;
use gui_widget::*;

#[derive(Debug)]
pub struct CameraSys {
  pub position: Vec3,
  pub projection: Projection,
  pub viewport: PhysicalSize,
  pub panning_speed: f32,
  pub show_debug_gui: bool,

  view_projection: Mat4,
  view_projection_inverse: Mat4,

  last_mouse_pos: Option<Vec2>,
}

#[derive(Debug)]
pub enum Projection {
  Orthographic {
    zoom_factor: f32,
    magnification_speed: f32,
    near: f32,
    far: f32,
  },
  Perspective {
    vertical_fov_degrees: f32,
    near: f32,
    far: f32,
  },
}

impl CameraSys {
  pub fn new(
    position: Vec3,
    projection: Projection,
    viewport: PhysicalSize,
    panning_speed: f32,
  ) -> CameraSys {
    CameraSys {
      position,
      projection,
      viewport,
      panning_speed,
      show_debug_gui: false,
      view_projection: Mat4::identity(),
      view_projection_inverse: Mat4::identity().inversed(),
      last_mouse_pos: None,
    }
  }

  pub fn with_defaults_perspective(viewport: PhysicalSize) -> CameraSys {
    CameraSys::new(DEFAULT_POSITION, DEFAULT_PERSPECTIVE, viewport, DEFAULT_PANNING_SPEED)
  }

  pub fn with_defaults_orthographic(viewport: PhysicalSize) -> CameraSys {
    CameraSys::new(DEFAULT_POSITION, DEFAULT_ORTHOGRAPHIC, viewport, DEFAULT_PANNING_SPEED)
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
    if let Projection::Orthographic { zoom_factor, magnification_speed, .. } = &mut self.projection {
      *zoom_factor *= 1.0 - input.zoom_delta * (*magnification_speed);
    }

    let (width, height): (f64, f64) = self.viewport.into();
    let width = width as f32;
    let height = height as f32;

    // View matrix.
    let eye = Vec3::new(self.position.x, self.position.y, self.position.z);
    let at = Vec3::new(self.position.x, self.position.y, self.position.z - 1.0);
    let up = Vec3::unit_y();
    let view = Mat4::look_at_lh(eye, at, up);

    // Projection matrix
    let aspect_ratio = width / height;
    let projection = match self.projection {
      Projection::Orthographic { zoom_factor, near, far, .. } => {
        let left = aspect_ratio * zoom_factor / -2.0;
        let right = aspect_ratio * zoom_factor / 2.0;
        let bottom = zoom_factor / -2.0;
        let top = zoom_factor / 2.0;
        projection::lh_yup::orthographic_wgpu_dx(left, right, bottom, top, near, far)
      }
      Projection::Perspective { vertical_fov_degrees, near, far } => {
        projection::lh_yup::perspective_wgpu_dx(vertical_fov_degrees.to_radians(), aspect_ratio, near, far)
      }
    };

    let view_projection = projection * view;
    self.view_projection = view_projection;
    self.view_projection_inverse = view_projection.inversed();

    if !self.show_debug_gui { return; }

    gui_context.window("Debug Camera", |ui| {
      ui.collapsing_open("Position", |ui| {
        ui.grid("Grid", |ui| {
          ui.label("Movement");
          ui.horizontal(|ui| {
            if input.move_up { ui.label("Up"); }
            if input.move_right { ui.label("Right"); }
            if input.move_down { ui.label("Down"); }
            if input.move_left { ui.label("Left"); }
          });
          ui.end_row();
          ui.label("Panning Speed");
          ui.show_f32(panning_speed);
          ui.end_row();
          ui.label("Position");
          ui.drag_vec3(0.1, &mut self.position);
          ui.end_row();
        });
      });
      CollapsingHeader::new("View").default_open(true).show(ui, |ui| {
        Grid::new("Grid")
          .striped(true)
          .spacing([10.0, 4.0])
          .show(ui, |ui| {
            ui.label("Eye");
            ui.show_vec3(&eye);
            ui.end_row();
            ui.label("At");
            ui.show_vec3(&at);
            ui.end_row();
            ui.label("Up");
            ui.show_vec3(&up);
            ui.end_row();
            ui.label("View matrix");
            ui.show_mat4(&view);
            ui.end_row();
          });
      });
    });
  }
}

// TODO: why is z 1.0? Shouldn't Z be -1.0, since 1.0 z is going INTO the screen? Is it because the view transformation is applied BEFORE the projection transformation, which flips the Z around?
const DEFAULT_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const DEFAULT_PANNING_SPEED: f32 = 10.0;
const DEFAULT_NEAR: f32 = 0.01;
const DEFAULT_FAR: f32 = 1000.0;
const DEFAULT_VERTICAL_FOV_DEGREES: f32 = 45.0;
const DEFAULT_PERSPECTIVE: Projection = Projection::Perspective { vertical_fov_degrees: DEFAULT_VERTICAL_FOV_DEGREES, near: DEFAULT_NEAR, far: DEFAULT_FAR };
const DEFAULT_ZOOM_FACTOR: f32 = 1.0;
const DEFAULT_MAGNIFICATION_SPEED: f32 = 0.1;
const DEFAULT_ORTHOGRAPHIC: Projection = Projection::Orthographic { zoom_factor: DEFAULT_ZOOM_FACTOR, magnification_speed: DEFAULT_MAGNIFICATION_SPEED, near: DEFAULT_NEAR, far: DEFAULT_FAR };

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

