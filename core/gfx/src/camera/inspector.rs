use egui::{Align2, ComboBox, Ui, Window};

use gui::Gui;
use gui::reset::UiResetButtonExt;
use gui::widget::UiWidgetsExt;

use crate::camera::controller::{CameraController, CameraControllerSettings, CameraControllerState, ControlType};
use crate::camera::system::CameraSystem;
use crate::camera::projection::{CameraProjection, CameraProjectionSettings, ProjectionType};

#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CameraInspector {
  pub show_window: bool,
  pub window_anchor: Option<Align2>,
}

impl CameraInspector {
  pub fn show_window(&mut self, gui: &Gui, system: &mut CameraSystem) {
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
            .selected_text(format!("Camera #{}", system.active_camera_index_mut()))
            .show_ui(ui, |ui| {
              for i in 0..system.camera_count() {
                ui.selectable_value(system.active_camera_index_mut(), i, format!("Camera #{}", i));
              }
            });
          ui.select_align2(&mut self.window_anchor);
          let default_data = system.default_data();
          let combined = system.active_camera();
          ui.reset_button()
            .compare(combined.state, default_data.state)
            .compare(combined.settings, default_data.settings)
            .reset_on_double_click();
        });
        let default_data = system.default_data();
        let combined = system.active_camera();
        Self::draw_controller(ui, &default_data.state.controller, &default_data.settings.controller, &mut combined.camera.controller, &mut combined.state.controller, &mut combined.settings.controller);
        Self::draw_projection(ui, &default_data.settings.projection, &mut combined.camera.projection, &mut combined.settings.projection);
      });
  }

  pub fn add_to_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.show_window, "Camera");
  }


  fn draw_controller(
    ui: &mut Ui,
    default_state: &CameraControllerState,
    default_settings: &CameraControllerSettings,
    controller: &mut CameraController,
    data: &mut CameraControllerState,
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
            ui.reset_button()
              .compare(&mut settings.arcball.mouse_movement_panning_speed, default_settings.arcball.mouse_movement_panning_speed)
              .compare(&mut settings.arcball.keyboard_panning_speed, default_settings.arcball.keyboard_panning_speed)
              .compare(&mut settings.arcball.debug_gui_panning_speed, default_settings.arcball.debug_gui_panning_speed)
              .reset_on_click();
          });
          ui.end_row();

          ui.label("Distance");
          ui.drag_unlabelled_range_with_reset(&mut data.arcball.distance, settings.arcball.debug_gui_distance_speed, 0.1..=f32::INFINITY, default_state.arcball.distance);
          ui.end_row();

          ui.label("Distance change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_scroll_distance_speed, 0.1, 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_distance_speed, 1.0, 0.0..=f32::INFINITY);
            ui.reset_button()
              .compare(&mut settings.arcball.mouse_scroll_distance_speed, default_settings.arcball.mouse_scroll_distance_speed)
              .compare(&mut settings.arcball.debug_gui_distance_speed, default_settings.arcball.debug_gui_distance_speed)
              .reset_on_click();
          });
          ui.end_row();

          ui.label("Rotation");
          ui.horizontal(|ui| {
            ui.drag("x: ", &mut data.arcball.rotation_around_x, 0.01);
            ui.drag("y: ", &mut data.arcball.rotation_around_y, 0.01);
            ui.reset_button()
              .compare(&mut data.arcball.rotation_around_x, default_state.arcball.rotation_around_x)
              .compare(&mut data.arcball.rotation_around_y, default_state.arcball.rotation_around_y)
              .reset_on_click();
          });
          ui.end_row();

          ui.label("Rotation change");
          ui.horizontal(|ui| {
            ui.drag_range("mouse: ", &mut settings.arcball.mouse_movement_rotation_speed, 1., 0.0..=f32::INFINITY);
            ui.drag_range("drag: ", &mut settings.arcball.debug_gui_rotation_speed, 1.0, 0.0..=f32::INFINITY);
            ui.reset_button()
              .compare(&mut settings.arcball.mouse_movement_rotation_speed, default_settings.arcball.mouse_movement_rotation_speed)
              .compare(&mut settings.arcball.debug_gui_rotation_speed, default_settings.arcball.debug_gui_rotation_speed)
              .reset_on_click();
          });
          ui.end_row();
        }
      }
      ui.label("Position");
      ui.show_vec3(&controller.position());
      ui.end_row();
      ui.label("Target");
      ui.drag_vec3_with_reset(settings.arcball.debug_gui_panning_speed, &mut data.target, default_state.target);
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
            ui.reset_button()
              .compare(&mut settings.perspective.vertical_fov_radians, default_settings.perspective.vertical_fov_radians)
              .reset_on_click();
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
