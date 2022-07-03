use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;

use gfx::camera::{CameraDebugging, CameraSettings};
use voxel::uniform::LightSettings;

use crate::SurfaceNetsDebugging;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Config {
  pub camera_settings: CameraSettings,
  pub camera_debugging: CameraDebugging,
  pub light_settings: LightSettings,
  pub surface_nets_debugging: SurfaceNetsDebugging,
}

fn default_camera_settings() -> CameraSettings {
  let mut settings = CameraSettings::with_defaults_arcball_orthographic();
  settings.arcball.distance = 3.0;
  settings
}

fn default_light_settings() -> LightSettings {
  let mut settings = LightSettings::default();
  settings.uniform.ambient = 0.2;
  settings.uniform.color = Vec3::new(0.0, 0.5, 0.35);
  settings
}

impl Default for Config {
  fn default() -> Self {
    Self {
      camera_settings: default_camera_settings(),
      camera_debugging: CameraDebugging {
        window_anchor: Some(egui::Align2::LEFT_BOTTOM),
        default_settings: default_camera_settings(),
        ..Default::default()
      },
      light_settings: default_light_settings(),
      surface_nets_debugging: Default::default()
    }
  }
}

impl Config {
  pub fn update_default_camera_settings(&mut self) {
    self.camera_debugging.default_settings = default_camera_settings();
  }
}
