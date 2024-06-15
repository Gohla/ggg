use egui::{CollapsingHeader, Grid, menu, Ui};
use serde::{Deserialize, Serialize};

use common::input::RawInput;
use common::sampler::{EventSampler, ValueSampler};
use common::time::Offset;
use gui::Gui;
use gui::widget::UiWidgetsExt;

use crate::{Frame, Step};
use crate::run::{FrameEnd, StepEnd, Updates};

// Timing statistics

#[derive(Default)]
pub struct TimingStats {
  /// Elapsed time since application start
  pub elapsed: Offset,

  /// Frame #
  pub frame: u64,
  /// Frame duration sampler.
  pub frame_duration: ValueSampler<Offset>,

  /// Target step duration.
  pub target_step_duration: Offset,
  /// Accumulated lag before updates.
  pub accumulated_lag_before_updates: Offset,
  /// Target ratio before updates.
  pub target_ratio_before_updates: f64,

  /// Step #
  pub step: u64,
  /// Step duration sampler
  pub step_duration: ValueSampler<Offset>,
  /// Step rate sampler
  pub step_rate: EventSampler,

  /// Accumulated lag after updates.
  pub accumulated_lag_after_updates: Offset,
  /// Target ratio after updates.
  pub target_ratio_after_updates: f64,
}

impl TimingStats {
  pub fn new() -> TimingStats { TimingStats::default() }

  pub fn elapsed(&mut self, elapsed: Offset) {
    self.elapsed = elapsed;
  }

  pub fn frame_start(&mut self, frame: Frame) {
    self.frame = frame.frame;
  }
  pub fn frame_end(&mut self, frame_end: FrameEnd) {
    self.frame_duration.add(frame_end.duration);
  }

  pub fn update_start(&mut self, updates: &Updates) {
    self.target_step_duration = updates.target_duration();
    self.accumulated_lag_before_updates = updates.accumulated_lag();
    self.target_ratio_before_updates = updates.target_ratio();
  }
  pub fn step_start(&mut self, step: Step) {
    self.step = step.update;
  }
  pub fn step_end(&mut self, step_end: StepEnd) {
    self.step_duration.add(step_end.duration);
    self.step_rate.add_now();
  }
  pub fn update_end(&mut self, updates: &Updates) {
    self.accumulated_lag_after_updates = updates.accumulated_lag();
    self.target_ratio_after_updates = updates.target_ratio();
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
    gui: &Gui,
    timing_stats: &TimingStats,
  ) {
    if !self.show_timing_window { return; }
    let mut window = gui.window("Debug Timing");
    if let Some(anchor) = self.timing_window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window.open(&mut self.show_timing_window)
      .auto_sized()
      .show(gui, |ui| {
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
              ui.label(format!("{:7.3}s", timing_stats.elapsed.into_seconds()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Frame").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Frame #");
              ui.label(format!("{}", timing_stats.frame));
              ui.end_row();
              ui.label("Frame duration");
              ui.label(format!("{:7.3}ms", timing_stats.frame_duration.avg().into_milliseconds()));
              ui.end_row();
              ui.label("FPS");
              ui.label(format!("{:7.3}", 1.0 / timing_stats.frame_duration.avg().into_seconds()));
              ui.end_row();
            });
        });

        CollapsingHeader::new("Update").default_open(true).show(ui, |ui| {
          Grid::new("Grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
              ui.label("Target update step duration");
              ui.label(format!("{:7.3}ms", timing_stats.target_step_duration.into_milliseconds()));
              ui.end_row();
              let target_ups = 1.0 / timing_stats.target_step_duration.into_seconds();
              ui.label("Target UPS");
              ui.label(format!("{:7.3}", target_ups));
              ui.end_row();
              ui.label("Lag before updates");
              ui.label(format!("{:7.3}ms", timing_stats.accumulated_lag_before_updates.into_milliseconds()));
              ui.end_row();
              ui.label("Steps needed before updates");
              ui.label(format!("{:7.3}", timing_stats.target_ratio_before_updates));
              ui.end_row();

              ui.label("Step #");
              ui.label(format!("{}", timing_stats.step));
              ui.end_row();
              ui.label("Step duration");
              ui.label(format!("{:7.3}ms", timing_stats.step_duration.avg().into_milliseconds()));
              ui.end_row();
              let (ups, ups_rate) = {
                let duration = timing_stats.step_rate.duration();
                let tps = if let Some(duration) = duration {
                  let ticks = timing_stats.step_rate.num_samples();
                  ticks as f64 / duration.into_seconds()
                } else {
                  0.0
                };
                let tps_rate = tps / target_ups;
                (tps, tps_rate)
              };
              ui.label("UPS");
              ui.label(format!("{:7.3}", ups));
              ui.end_row();
              ui.label("UPS target rate");
              ui.label(format!("{:5.1}%", ups_rate * 100.0));
              ui.end_row();

              ui.label("Lag after updates");
              ui.label(format!("{:7.3}ms", timing_stats.accumulated_lag_after_updates.into_milliseconds()));
              ui.end_row();
              ui.label("Steps needed after updates");
              ui.label(format!("{:7.3}", timing_stats.target_ratio_after_updates));
              ui.end_row();
            });
        });
      });
  }

  #[profiling::function]
  pub fn show_input(
    &mut self,
    gui: &Gui,
    input: &RawInput,
  ) {
    if !self.show_input_window { return; }
    let mut window = gui.window("Debug Input");
    if let Some(anchor) = self.input_window_anchor {
      window = window.anchor(anchor, egui::Vec2::ZERO);
    }
    window.open(&mut self.show_input_window)
      .auto_sized()
      .show(gui, |ui| {
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
