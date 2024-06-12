use egui::{CollapsingHeader, Context, Grid, menu, Ui, Window};
use serde::{Deserialize, Serialize};

use common::input::RawInput;
use common::sampler::{EventSampler, ValueSampler};
use common::timing::{Instant, Offset};
use gui_widget::UiWidgetsExt;

use crate::Cycle;
use crate::run::StepEnd;

// Timing statistics

#[derive(Default)]
pub struct TimingStats {
  /// Elapsed time since application start
  pub elapsed: Offset,

  /// Cycle #
  pub cycle: u64,
  /// Cycle duration sampler
  pub cycle_duration: ValueSampler<Offset>,

  /// Step target duration
  pub step_target_duration: Offset,
  /// Step #
  pub step: u64,
  /// Step duration sampler
  pub step_duration: ValueSampler<Offset>,
  /// Step rate sampler
  pub step_rate: EventSampler,

  /// Accumulated lag
  pub accumulated_lag: Offset,
  /// Render extrapolation
  pub render_extrapolation: f64,
}

impl TimingStats {
  pub fn new() -> TimingStats { TimingStats::default() }

  pub fn elapsed(&mut self, elapsed: Offset) {
    self.elapsed = elapsed;
  }

  pub fn cycle(&mut self, cycle: Cycle) {
    self.cycle = cycle.cycle;
    self.cycle_duration.add(cycle.duration);
  }

  pub fn step(&mut self, step_end: StepEnd) {
    self.step_target_duration = step_end.target_duration;
    self.step = step_end.step;
    self.step_duration.add(step_end.duration);
    self.step_rate.add(Instant::now())
  }

  pub fn step_lag(&mut self, accumulated_lag: Offset, gfx_extrapolation: f64) {
    self.accumulated_lag = accumulated_lag;
    self.render_extrapolation = gfx_extrapolation;
  }
}

// Debug GUI

#[derive(Default, Copy, Clone, Serialize, Deserialize, Debug)]
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
      ui.separator();
      add_contents(ui);
      ui.separator();
      if ui.button("Reset GUI state (double click)").double_clicked() {
        ui.ctx().memory_mut(|m| *m = egui::Memory::default());
      }
    });
  }

  #[profiling::function]
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
              ui.label(format!("{:7.3}s", timing_stats.elapsed.as_s()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Cycle").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Cycle #");
              ui.label(format!("{}", timing_stats.cycle));
              ui.end_row();
              ui.label("Cycle duration");
              ui.label(format!("{:7.3}ms", timing_stats.cycle_duration.avg().as_ms()));
              ui.end_row();
              ui.label("CPS");
              ui.label(format!("{:7.3}", 1.0 / timing_stats.cycle_duration.avg().as_s()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Simulation update step").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Step target duration");
              ui.label(format!("{:7.3}ms", timing_stats.step_target_duration.as_ms()));
              ui.end_row();
              let sps_target = 1.0 / timing_stats.step_target_duration.as_s();
              ui.label("SPS target");
              ui.label(format!("{:7.3}", sps_target));
              ui.end_row();
              ui.label("Step #");
              ui.label(format!("{}", timing_stats.step));
              ui.end_row();
              ui.label("Step duration");
              ui.label(format!("{:7.3}ms", timing_stats.step_duration.avg().as_ms()));
              ui.end_row();
              let (sps, sps_rate) = {
                let duration = timing_stats.step_rate.duration();
                let tps = if let Some(duration) = duration {
                  let ticks = timing_stats.step_rate.num_samples();
                  ticks as f64 / duration.as_s()
                } else {
                  0.0
                };
                let tps_rate = tps / sps_target;
                (tps, tps_rate)
              };
              ui.label("SPS");
              ui.label(format!("{:7.3}", sps));
              ui.end_row();
              ui.label("SPS target rate");
              ui.label(format!("{:5.1}%", sps_rate * 100.0));
              ui.end_row();
              ui.label("Accumulated step lag");
              ui.label(format!("{:7.3}ms", timing_stats.accumulated_lag.as_ms()));
              ui.end_row();
              ui.label("Render extrapolation");
              ui.label(format!("{:5.1}%", timing_stats.render_extrapolation * 100.0));
              ui.end_row();
            });
        });
      });
  }

  #[profiling::function]
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
              ui.label("Keyboard Keys");
              ui.label(format!("{:?}", input.keyboard_keys));
              ui.end_row();
              ui.label("Semantic Keys");
              ui.label(format!("{:?}", input.semantic_keys));
              ui.end_row();
            });
        });
      });
  }
}
