use std::slice::{from_mut, from_ref};

use egui::{Align2, ComboBox, Ui, Window};

use gui_widget::{Gui, UiWidgetsExt};

use super::{Camera, CameraSettings, MovementType, ProjectionType};

#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraDebugging {
  pub show_window: bool,
  pub window_anchor: Option<Align2>,
  pub selected_camera: usize,
  #[cfg_attr(feature = "serde", serde(skip))]
  pub default_settings: CameraSettings,
}

impl CameraDebugging {
  pub fn with_default_settings(default_settings: CameraSettings) -> Self {
    Self {
      default_settings,
      ..CameraDebugging::default()
    }
  }

  pub fn show(&mut self, gui: &Gui, camera: &Camera, settings: &mut CameraSettings) {
    self.show_multiple_cameras(gui, from_ref(camera), from_mut(settings));
  }

  pub fn show_multiple_cameras(&mut self, gui: &Gui, cameras: &[Camera], settings: &mut [CameraSettings]) {
    if !self.show_window { return; }
    let mut window = Window::new("Camera")
      .constrain_to(gui.area_under_title_bar);
    if let Some(anchor) = self.window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window
      .open(&mut self.show_window)
      .auto_sized()
      .show(gui, |ui| Self::draw_debugging_gui(ui, &mut self.window_anchor, &mut self.selected_camera, &self.default_settings, cameras, settings));
  }

  pub fn add_to_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.show_window, "Camera");
  }

  pub fn get_selected_camera<'c>(&self, cameras: &'c [Camera]) -> &'c Camera {
    &cameras[self.selected_camera]
  }

  pub fn get_selected_camera_mut<'c>(&self, cameras: &'c mut [Camera]) -> &'c mut Camera {
    &mut cameras[self.selected_camera]
  }

  pub fn get_selected_camera_and_settings<'c>(&self, cameras: &'c mut [Camera], settings: &'c mut [CameraSettings]) -> (&'c mut Camera, &'c mut CameraSettings) {
    let camera = &mut cameras[self.selected_camera];
    let settings = &mut settings[self.selected_camera];
    (camera, settings)
  }

  fn draw_debugging_gui(
    ui: &mut Ui,
    window_anchor: &mut Option<Align2>,
    selected_camera: &mut usize,
    default_settings: &CameraSettings,
    cameras: &[Camera],
    settings: &mut [CameraSettings],
  ) {
    use egui::ComboBox;
    use gui_widget::*;
    ui.horizontal(|ui| {
      ComboBox::from_id_source("Camera")
        .selected_text(format!("Camera #{}", selected_camera))
        .show_ui(ui, |ui| {
          for i in 0..cameras.len() {
            ui.selectable_value(selected_camera, i, format!("Camera #{}", i));
          }
        });
      ui.select_align2(window_anchor);
      ui.reset_button_double_click_with(&mut settings[*selected_camera], *default_settings);
    });
    let camera = &cameras[*selected_camera];
    let settings = &mut settings[*selected_camera];
    Self::draw_debugging_gui_for_camera(ui, default_settings, camera, settings);
  }

  fn draw_debugging_gui_for_camera(
    ui: &mut Ui,
    default_settings: &CameraSettings,
    camera: &Camera,
    settings: &mut CameraSettings,
  ) {
    ui.collapsing_open_with_grid("View", "Grid", |ui| {
      ui.label("Movement mode");
      ComboBox::from_id_source("Movement mode")
        .selected_text(format!("{:?}", settings.movement_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut settings.movement_type, MovementType::Arcball, "Arcball");
          ui.selectable_value(&mut settings.movement_type, MovementType::Fly, "Fly");
        });
      ui.end_row();
      match settings.movement_type {
        MovementType::Arcball => {
          ui.label("Panning change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_movement_panning_speed, 1.0, 0.0..=f32::INFINITY);
            ui.drag_range("key: ", &mut settings.arcball.keyboard_panning_speed, 1.0, 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_panning_speed, 1.0, 0.0..=f32::INFINITY);
            let can_reset = settings.arcball.mouse_movement_panning_speed != default_settings.arcball.mouse_movement_panning_speed
              || settings.arcball.keyboard_panning_speed != default_settings.arcball.keyboard_panning_speed
              || settings.arcball.debug_gui_panning_speed != default_settings.arcball.debug_gui_panning_speed;
            if ui.reset_button_response(can_reset).clicked() {
              settings.arcball.mouse_movement_panning_speed = default_settings.arcball.mouse_movement_panning_speed;
              settings.arcball.keyboard_panning_speed = default_settings.arcball.keyboard_panning_speed;
              settings.arcball.debug_gui_panning_speed = default_settings.arcball.debug_gui_panning_speed;
            }
          });
          ui.end_row();
          ui.label("Distance");
          ui.drag_unlabelled_range_with_reset(&mut settings.arcball.distance, settings.arcball.debug_gui_distance_speed, 0.1..=f32::INFINITY, default_settings.arcball.distance);
          ui.end_row();
          ui.label("Distance change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_scroll_distance_speed, 0.1, 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_distance_speed, 1.0, 0.0..=f32::INFINITY);
            if ui.reset_button_response(settings.arcball.mouse_scroll_distance_speed != default_settings.arcball.mouse_scroll_distance_speed || settings.arcball.debug_gui_distance_speed != default_settings.arcball.debug_gui_distance_speed).clicked() {
              settings.arcball.mouse_scroll_distance_speed = default_settings.arcball.mouse_scroll_distance_speed;
              settings.arcball.debug_gui_distance_speed = default_settings.arcball.debug_gui_distance_speed;
            }
          });
          ui.end_row();
          ui.label("Rotation");
          ui.horizontal(|ui| {
            ui.drag("x: ", &mut settings.arcball.rotation_around_x, 0.01);
            ui.drag("y: ", &mut settings.arcball.rotation_around_y, 0.01);
            if ui.reset_button_response(settings.arcball.rotation_around_x != default_settings.arcball.rotation_around_x || settings.arcball.rotation_around_y != default_settings.arcball.rotation_around_y).clicked() {
              settings.arcball.rotation_around_x = default_settings.arcball.rotation_around_x;
              settings.arcball.rotation_around_y = default_settings.arcball.rotation_around_y;
            }
          });
          ui.end_row();
          ui.label("Rotation change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_movement_rotation_speed, 1., 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_rotation_speed, 1.0, 0.0..=f32::INFINITY);
            if ui.reset_button_response(settings.arcball.mouse_movement_rotation_speed != default_settings.arcball.mouse_movement_rotation_speed || settings.arcball.debug_gui_rotation_speed != default_settings.arcball.debug_gui_rotation_speed).clicked() {
              settings.arcball.mouse_movement_rotation_speed = default_settings.arcball.mouse_movement_rotation_speed;
              settings.arcball.debug_gui_rotation_speed = default_settings.arcball.debug_gui_rotation_speed;
            }
          });
          ui.end_row();
        }
        MovementType::Fly => {}
      }
      ui.label("Target");
      ui.drag_vec3_with_reset(settings.arcball.debug_gui_panning_speed, &mut settings.target, default_settings.target);
      ui.end_row();
      ui.label("Position");
      ui.show_vec3(&camera.position);
      ui.end_row();
    });
    ui.collapsing_open_with_grid("Projection", "Grid", |ui| {
      ui.label("Projection mode");
      ComboBox::from_id_source("Projection mode")
        .selected_text(format!("{:?}", settings.projection_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut settings.projection_type, ProjectionType::Perspective, "Perspective");
          ui.selectable_value(&mut settings.projection_type, ProjectionType::Orthographic, "Orthographic");
        });
      ui.end_row();
      match settings.projection_type {
        ProjectionType::Perspective => {
          ui.label("Vertical FOV");
          ui.drag_angle(&mut settings.perspective.vertical_fov_radians);
          ui.end_row();
        }
        ProjectionType::Orthographic => {}
      }
      ui.label("Viewport");
      ui.monospace(format!("{:.2}x{:.2} ({:.2})", camera.viewport.width, camera.viewport.height, camera.viewport.ratio()));
      ui.end_row();
      ui.label("Frustum");
      ui.horizontal(|ui| {
        ui.drag("near: ", &mut settings.near, 0.001);
        ui.drag("far: ", &mut settings.far, 1.0);
      });
      ui.end_row();
    });
    ui.collapsing_with_grid("Matrices", "Grid", |ui| {
      ui.label("View matrix");
      ui.show_mat4(&camera.view);
      ui.end_row();
      ui.label("Projection matrix");
      ui.show_mat4(&camera.projection);
      ui.end_row();
      ui.label("View-projection matrix");
      ui.show_mat4(&camera.view_projection);
      ui.end_row();
      ui.label("View-projection matrix inverse");
      ui.show_mat4(&camera.view_projection_inverse);
      ui.end_row();
    });
  }
}
