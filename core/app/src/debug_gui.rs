use egui::{CollapsingHeader, CtxRef, Grid, menu, Ui, Window};

use common::timing::TimingStats;

#[derive(Default)]
pub struct DebugGui {
  show_timing: bool,
  show_input: bool,
}

impl DebugGui {
  pub fn add_debug_menu(&mut self, ui: &mut Ui) {
    menu::menu(ui, "Debug", |ui| {
      ui.checkbox(&mut self.show_timing, "Timing");
      ui.checkbox(&mut self.show_input, "Input");
    });
  }

  pub fn show_timing(
    &mut self,
    ctx: &CtxRef,
    timing_stats: &TimingStats,
  ) {
    if !self.show_timing { return; }
    Window::new("Timing").show(ctx, |ui| {
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
}
