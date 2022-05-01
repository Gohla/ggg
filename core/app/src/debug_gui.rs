use egui::{CollapsingHeader, Context, Grid, menu, Ui, Window};

use common::input::RawInput;
use common::timing::TimingStats;
use gui_widget::UiWidgetsExt;

#[derive(Default, Copy, Clone, Debug)]
pub struct DebugGui {
  pub show_timing_window: bool,
  pub timing_window_anchor: Option<egui::Align2>,
  pub show_input_window: bool,
  pub input_window_anchor: Option<egui::Align2>,
}

impl DebugGui {
  pub fn add_debug_menu(&mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    menu::menu_button(ui, "Debug", |ui| {
      ui.checkbox(&mut self.show_timing_window, "Timing");
      ui.checkbox(&mut self.show_input_window, "Input");
      add_contents(ui);
    });
  }

  pub fn show_timing(
    &mut self,
    ctx: &Context,
    timing_stats: &TimingStats,
  ) {
    if !self.show_timing_window { return; }
    let mut window = Window::new("Debug Timing");
    if let Some(anchor) = self.timing_window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window.open(&mut self.show_timing_window)
      .auto_sized()
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.label("Anchor");
          ui.select_align2(&mut self.timing_window_anchor);
        });
        CollapsingHeader::new("Time").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Elapsed");
              ui.label(format!("{:7.3}s", timing_stats.elapsed_time.as_s()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Frame").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Frame #");
              ui.label(format!("{}", timing_stats.frame_count));
              ui.end_row();
              ui.label("Frame time");
              ui.label(format!("{:7.3}ms", timing_stats.frame_time.avg().as_ms()));
              ui.end_row();
              ui.label("FPS");
              ui.label(format!("{:7.3}", 1.0 / timing_stats.frame_time.avg().as_s()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Tick").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Tick #");
              ui.label(format!("{}", timing_stats.tick_count));
              ui.end_row();
              ui.label("Tick time target");
              ui.label(format!("{:7.3}ms", timing_stats.tick_time_target.as_ms()));
              ui.end_row();
              let tps_target = 1.0 / timing_stats.tick_time_target.as_s();
              ui.label("TPS target");
              ui.label(format!("{:7.3}", tps_target));
              ui.end_row();
              ui.label("Tick time");
              ui.label(format!("{:7.3}ms", timing_stats.tick_time.avg().as_ms()));
              ui.end_row();
              let (tps, tps_rate) = {
                let duration = timing_stats.tick_rate.duration();
                let tps = if let Some(duration) = duration {
                  let ticks = timing_stats.tick_rate.num_samples();
                  ticks as f64 / duration.as_s()
                } else {
                  0.0
                };
                let tps_rate = tps / tps_target;
                (tps, tps_rate)
              };
              ui.label("TPS");
              ui.label(format!("{:7.3}", tps));
              ui.end_row();
              ui.label("TPS rate");
              ui.label(format!("{:5.1}%", tps_rate * 100.0));
              ui.end_row();
              ui.label("Accumulated tick lag");
              ui.label(format!("{:7.3}ms", timing_stats.accumulated_lag.as_ms()));
              ui.end_row();
              ui.label("Render extrapolation");
              ui.label(format!("{:5.1}%", timing_stats.render_extrapolation * 100.0));
              ui.end_row();
            });
        });
      });
  }

  pub fn show_input(
    &mut self,
    ctx: &Context,
    input: &RawInput,
  ) {
    if !self.show_input_window { return; }
    let mut window = Window::new("Debug Input");
    if let Some(anchor) = self.input_window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window.open(&mut self.show_input_window)
      .auto_sized()
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.label("Anchor");
          ui.select_align2(&mut self.input_window_anchor);
        });
        CollapsingHeader::new("Mouse").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Buttons");
              ui.label(format!("{:?}", input.mouse_buttons));
              ui.end_row();
              ui.label("Physical position");
              ui.label(format!("x: {}, y: {}", input.mouse_position.physical.x, input.mouse_position.physical.y));
              ui.end_row();
              ui.label("Logical position");
              ui.label(format!("x: {}, y: {}", input.mouse_position.logical.x, input.mouse_position.logical.y));
              ui.end_row();
            });
        });
        CollapsingHeader::new("Keyboard").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Modifiers");
              ui.label(format!("{:?}", input.keyboard_modifiers));
              ui.end_row();
              ui.label("Buttons");
              ui.label(format!("{:?}", input.keyboard_buttons));
              ui.end_row();
            });
        });
      });
  }
}
