use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;

use gfx::camera::{CameraData, CameraSettings};
use gfx::camera::inspector::CameraInspector;
use gfx::camera::projection::ProjectionType;
use voxel::uniform::LightSettings;

use crate::inspector::Inspector;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  pub camera_data: CameraData,
  pub camera_settings: CameraSettings,
  pub camera_inspector: CameraInspector,
  pub light: LightSettings,
  pub inspector: Inspector,
}
impl Default for Data {
  fn default() -> Self {
    let mut data = Self {
      camera_data: Default::default(),
      camera_settings: Default::default(),
      camera_inspector: Default::default(),
      light: Default::default(),
      inspector: Default::default(),
    };
    data.camera_settings.projection.projection_type = ProjectionType::Orthographic;
    data.camera_data.controller.arcball.distance = 2.0;

    data.light.uniform.ambient = 0.2;
    data.light.uniform.color = Vec3::new(0.0, 0.5, 0.35);

    data.camera_inspector.default_data = data.camera_data;
    data.camera_inspector.default_settings = data.camera_settings;
    data
  }
}
impl Data {
  pub fn set_camera_inspector_defaults(&mut self) {
    let default = Self::default();
    self.camera_inspector.default_data = default.camera_data;
    self.camera_inspector.default_settings = default.camera_settings;
  }
}
