use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;

use gfx::camera::inspector::CameraInspector;
use gfx::camera::system::CameraSystemState;
use voxel::uniform::LightSettings;

use crate::inspector::Inspector;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  pub camera_manager_state: CameraSystemState,
  pub camera_inspector: CameraInspector,
  pub light: LightSettings,
  pub inspector: Inspector,
}
impl Default for Data {
  fn default() -> Self {
    let mut data = Self {
      camera_manager_state: Default::default(),
      camera_inspector: Default::default(),
      light: Default::default(),
      inspector: Default::default(),
    };
    data.light.uniform.ambient = 0.2;
    data.light.uniform.color = Vec3::new(0.0, 0.5, 0.35);
    data
  }
}
