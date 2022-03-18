use egui::Align2;

use app::GuiFrame;

#[derive(Default)]
pub struct TransvoxelDebugging {
  _equivalence_class: u8,
}

impl TransvoxelDebugging {
  pub fn render_gui(&mut self, gui_frame: &GuiFrame) {
    egui::Window::new("Transvoxel")
      .anchor(Align2::RIGHT_TOP, egui::Vec2::default())
      .show(&gui_frame, |_ui| {

      });
  }
}
