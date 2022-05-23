use bytemuck::{Pod, Zeroable};
use egui::{CollapsingResponse, color_picker, DragValue, InnerResponse, Rgba, Ui};
use egui::color_picker::Alpha;
use ultraviolet::{Isometry3, Mat4, Rotor3, Vec3, Vec4};

use gfx::camera::Camera;
use gui_widget::UiWidgetsExt;

// Camera

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable, Debug)]
pub struct CameraUniform {
  pub position: Vec4,
  pub view_projection: Mat4,
}

impl CameraUniform {
  pub fn from_camera(camera: &Camera) -> Self {
    Self {
      position: camera.get_position().into_homogeneous_point(),
      view_projection: camera.get_view_projection_matrix(),
    }
  }

  pub fn update_from_camera(&mut self, camera: &Camera) {
    self.position = camera.get_position().into_homogeneous_point();
    self.view_projection = camera.get_view_projection_matrix();
  }
}

// Light

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LightUniform {
  pub color: Vec3,
  pub ambient: f32,
  pub direction: Vec3,
  _dummy: f32, // TODO: replace with crevice crate?
}

impl LightUniform {
  pub fn new(color: Vec3, ambient: f32, direction: Vec3) -> Self {
    Self { color, ambient, direction, _dummy: 0.0 }
  }
}

impl Default for LightUniform {
  fn default() -> Self {
    Self::new(Vec3::new(0.9, 0.9, 0.9), 0.01, Vec3::new(-0.5, -0.5, -0.5))
  }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LightSettings {
  pub rotation_x_degree: f32,
  pub rotation_y_degree: f32,
  pub rotation_z_degree: f32,
  pub follow_camera: bool,
  pub uniform: LightUniform,
}

impl Default for LightSettings {
  fn default() -> Self {
    Self {
      rotation_x_degree: 0.0,
      rotation_y_degree: 270.0,
      rotation_z_degree: 0.0,
      follow_camera: false,
      uniform: Default::default(),
    }
  }
}

impl LightSettings {
  pub fn render_gui(&mut self, ui: &mut Ui, camera: &Camera) -> CollapsingResponse<InnerResponse<()>> {
    ui.collapsing_open_with_grid("Directional Light", "Grid", |mut ui| {
      ui.label("Color");
      let mut color = Rgba::from_rgba_premultiplied(self.uniform.color.x, self.uniform.color.y, self.uniform.color.z, 0.0).into();
      color_picker::color_edit_button_srgba(&mut ui, &mut color, Alpha::Opaque);
      let color: Rgba = color.into();
      self.uniform.color = Vec3::new(color.r(), color.g(), color.b());
      ui.end_row();
      ui.label("Ambient");
      ui.add(DragValue::new(&mut self.uniform.ambient).speed(0.001).clamp_range(0.0..=1.0));
      ui.end_row();
      ui.label("Follow camera?");
      ui.checkbox(&mut self.follow_camera, "");
      if self.follow_camera {
        self.uniform.direction = camera.get_direction_inverse();
      } else {
        ui.label("Direction");
        ui.drag("x: ", &mut self.rotation_x_degree, 0.5);
        ui.drag("y: ", &mut self.rotation_y_degree, 0.5);
        ui.drag("z: ", &mut self.rotation_z_degree, 0.5);
        self.uniform.direction = Rotor3::from_euler_angles((self.rotation_z_degree % 360.0).to_radians(), (self.rotation_x_degree % 360.0).to_radians(), (self.rotation_y_degree % 360.0).to_radians()) * Vec3::one();
      }
      ui.end_row();
    })
  }
}

// Model

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable, Debug)]
pub struct ModelUniform {
  pub model: Mat4,
}

impl ModelUniform {
  #[inline]
  pub fn new(model: Mat4) -> Self { Self { model } }

  #[inline]
  pub fn from_transform(transform: Isometry3) -> Self { Self::new(transform.into_homogeneous_matrix()) }

  #[inline]
  pub fn identity() -> Self { Self::from_transform(Isometry3::identity()) }
}
