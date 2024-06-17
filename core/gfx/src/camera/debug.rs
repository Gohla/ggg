use std::slice::from_mut;

use egui::{Align2, ComboBox, Ui, Window};

use gui::Gui;
use gui::widget::UiWidgetsExt;

use crate::camera::controller::{CameraController, CameraControllerData, CameraControllerSettings, ControlType};
use crate::camera::projection::{CameraProjection, CameraProjectionSettings, ProjectionType};

use super::{Camera, CameraData, CameraSettings};

#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraDebugging {
  pub show_window: bool,
  pub window_anchor: Option<Align2>,
  pub selected_camera: usize,

  #[cfg_attr(feature = "serde", serde(skip))]
  pub default_data: CameraData,
  #[cfg_attr(feature = "serde", serde(skip))]
  pub default_settings: CameraSettings,
}

impl CameraDebugging {
  pub fn default_data(mut self, default_data: CameraData) -> Self {
    self.default_data = default_data;
    self
  }
  pub fn default_settings(mut self, default_settings: CameraSettings) -> Self {
    self.default_settings = default_settings;
    self
  }

  pub fn selected_camera<'c>(&self, cameras: &'c [Camera]) -> &'c Camera {
    &cameras[self.selected_camera]
  }
  pub fn selected_camera_mut<'c>(&self, cameras: &'c mut [Camera]) -> &'c mut Camera {
    &mut cameras[self.selected_camera]
  }
  pub fn selected<'c>(&self, cameras: &'c mut [Camera],  data: &'c mut [CameraData], settings: &'c [CameraSettings]) -> (&'c mut Camera, &'c mut CameraData, &'c CameraSettings) {
    let camera = &mut cameras[self.selected_camera];
    let data = &mut data[self.selected_camera];
    let settings = &settings[self.selected_camera];
    (camera, data, settings)
  }


  pub fn show_single(&mut self, gui: &Gui, camera: &mut Camera, data: &mut CameraData, settings: &mut CameraSettings) {
    self.show(gui, from_mut(camera), from_mut(data), from_mut(settings));
  }
  pub fn show(&mut self, gui: &Gui, cameras: &mut [Camera], data: &mut [CameraData], settings: &mut [CameraSettings]) {
    if !self.show_window { return; }
    let mut window = Window::new("Camera")
      .constrain_to(gui.area_under_title_bar);
    if let Some(anchor) = self.window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window
      .open(&mut self.show_window)
      .auto_sized()
      .show(gui, |ui| {
        ui.horizontal(|ui| {
          ComboBox::from_id_source("Camera")
            .selected_text(format!("Camera #{}", self.selected_camera))
            .show_ui(ui, |ui| {
              for i in 0..cameras.len() {
                ui.selectable_value(&mut self.selected_camera, i, format!("Camera #{}", i));
              }
            });
          ui.select_align2(&mut self.window_anchor);
          ui.reset_button_double_click_with(&mut (&mut settings[self.selected_camera], &mut data[self.selected_camera]), (&mut self.default_settings, &mut self.default_data));
        });
        let camera = &mut cameras[self.selected_camera];
        let data = &mut data[self.selected_camera];
        let settings = &mut settings[self.selected_camera];
        Self::draw_controller(ui, &self.default_data.controller, &self.default_settings.controller, &mut camera.controller, &mut data.controller, &mut settings.controller);
        Self::draw_projection(ui, &self.default_settings.projection, &mut camera.projection, &mut settings.projection);
      });
  }

  pub fn add_to_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.show_window, "Camera");
  }


  fn draw_controller(
    ui: &mut Ui,
    default_data: &CameraControllerData,
    default_settings: &CameraControllerSettings,
    controller: &mut CameraController,
    data: &mut CameraControllerData,
    settings: &mut CameraControllerSettings,
  ) {
    ui.collapsing_open_with_grid("Controller", "Grid", |ui| {
      ui.label("Control type");
      ComboBox::from_id_source("Control type")
        .selected_text(format!("{:?}", settings.control_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut settings.control_type, ControlType::Arcball, "Arcball");
        });
      ui.end_row();
      match settings.control_type {
        ControlType::Arcball => {
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
          ui.drag_unlabelled_range_with_reset(&mut data.arcball.distance, settings.arcball.debug_gui_distance_speed, 0.1..=f32::INFINITY, default_data.arcball.distance);
          ui.end_row();

          ui.label("Distance change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_scroll_distance_speed, 0.1, 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_distance_speed, 1.0, 0.0..=f32::INFINITY);
            let can_reset = settings.arcball.mouse_scroll_distance_speed != default_settings.arcball.mouse_scroll_distance_speed
              || settings.arcball.debug_gui_distance_speed != default_settings.arcball.debug_gui_distance_speed;
            if ui.reset_button_response(can_reset).clicked() {
              settings.arcball.mouse_scroll_distance_speed = default_settings.arcball.mouse_scroll_distance_speed;
              settings.arcball.debug_gui_distance_speed = default_settings.arcball.debug_gui_distance_speed;
            }
          });
          ui.end_row();

          ui.label("Rotation");
          ui.horizontal(|ui| {
            ui.drag("x: ", &mut data.arcball.rotation_around_x, 0.01);
            ui.drag("y: ", &mut data.arcball.rotation_around_y, 0.01);
            let can_reset = data.arcball.rotation_around_x != default_data.arcball.rotation_around_x
              || data.arcball.rotation_around_y != default_data.arcball.rotation_around_y;
            if ui.reset_button_response(can_reset).clicked() {
              data.arcball.rotation_around_x = default_data.arcball.rotation_around_x;
              data.arcball.rotation_around_y = default_data.arcball.rotation_around_y;
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
      }
      ui.label("Position");
      ui.show_vec3(&controller.position());
      ui.end_row();
      ui.label("Target");
      ui.drag_vec3_with_reset(settings.arcball.debug_gui_panning_speed, &mut data.target, default_data.target);
      ui.end_row();
    });
  }

  fn draw_projection(
    ui: &mut Ui,
    default_settings: &CameraProjectionSettings,
    projection: &mut CameraProjection,
    settings: &mut CameraProjectionSettings,
  ) {
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
          ui.horizontal(|ui| {
            ui.drag_angle(&mut settings.perspective.vertical_fov_radians);
            ui.reset_button_with(&mut settings.perspective.vertical_fov_radians, default_settings.perspective.vertical_fov_radians);
          });
          ui.end_row();
        }
        ProjectionType::Orthographic => {}
      }
      ui.label("Viewport");
      let viewport = projection.viewport();
      ui.monospace(format!("{:.2}x{:.2} ({:.2})", viewport.width, viewport.height, viewport.ratio()));
      ui.end_row();

      ui.label("Frustum");
      ui.horizontal(|ui| {
        ui.drag_with_reset("near: ", &mut settings.near, 0.001, default_settings.near);
        ui.drag_with_reset("far: ", &mut settings.far, 1.0, default_settings.far);
      });
      ui.end_row();
    });
    ui.collapsing_with_grid("Matrices", "Grid", |ui| {
      ui.label("View");
      ui.show_mat4(projection.view_matrix());
      ui.end_row();
      ui.label("Projection");
      ui.show_mat4(projection.projection_matrix());
      ui.end_row();
      ui.label("View-projection");
      ui.show_mat4(projection.view_projection_matrix());
      ui.end_row();
      ui.label("View-projection inverse");
      ui.show_mat4(projection.inverse_view_projection_matrix());
      ui.end_row();
    });
  }
}
