use egui::{Align2, Ui};
use serde::{Deserialize, Serialize};
use ultraviolet::{UVec3, Vec4};

use app::GuiFrame;
use gfx::debug_renderer::DebugRenderer;
use gfx::display_math::{UVec3DisplayExt, Vec3DisplayExt};
use gui_widget::UiWidgetsExt;
use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::sample::{ChunkSampleArray, ChunkSamples};
use voxel::chunk::size::ChunkSize;
use voxel::surface_nets::{Cell, SurfaceNets};

use crate::C2;

pub const MIN: UVec3 = UVec3::new(0, 0, 0);
pub const STEP: u32 = 1;

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct SurfaceNetsDebugging {
  samples: ChunkSampleArray<C2>,
  selected_cell: Cell,
}

impl SurfaceNetsDebugging {
  pub fn show_gui_window(&mut self, gui_frame: &GuiFrame) {
    egui::Window::new("Surface Nets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        self.draw_window_contents(ui);
      });
  }

  fn draw_window_contents(&mut self, ui: &mut Ui) {
    self.draw_cell_gui(ui);
    self.draw_data_gui(ui);
  }

  fn draw_cell_gui(&mut self, ui: &mut Ui) {
    ui.collapsing_open("Cell", |ui| {
      ui.horizontal(|ui| {
        if ui.button("Flip").clicked() {
          self.samples.flip_all();
        }
        if ui.button("To +0.0").clicked() {
          self.samples.set_all_to(0.0);
        }
        if ui.button("To -0.0").clicked() {
          self.samples.set_all_to(-0.0);
        }
        if ui.button("To +1.0").clicked() {
          self.samples.set_all_to(1.0);
        }
        if ui.button("To -1.0").clicked() {
          self.samples.set_all_to(-1.0);
        }
        ui.end_row();
      });
      for z in 0..C2::VOXELS_IN_CHUNK_ROW {
        ui.collapsing_open_with_grid(format!("Z={}", z), format!("Grid Z={}", z), |ui| {
          ui.label("");
          for x in 0..C2::VOXELS_IN_CHUNK_ROW {
            ui.label(format!("{}", x));
          }
          ui.end_row();
          for y in (0..C2::VOXELS_IN_CHUNK_ROW).rev() {
            ui.label(format!("Y={}", y));
            for x in 0..C2::VOXELS_IN_CHUNK_ROW {
              let sample = self.samples.sample_mut(UVec3::new(x, y, z));
              let response = ui.drag("", sample, 0.01);
              if response.secondary_clicked() {
                *sample = 0.0;
              }
              if response.middle_clicked() {
                *sample *= -1.0;
              }
            }
            ui.end_row();
          }
        });
      }
    });
  }

  fn draw_data_gui(&mut self, ui: &mut Ui) {
    ui.collapsing_open("Cell Data", |ui| {
      ui.horizontal(|ui| {
        ui.label("Selected Cell");
        ui.drag_range("x: ", &mut self.selected_cell.x, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
        ui.drag_range("y: ", &mut self.selected_cell.y, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
        ui.drag_range("z: ", &mut self.selected_cell.z, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
      });

      let local_voxel_positions = SurfaceNets::<C2>::local_voxel_positions(self.selected_cell);
      let values = SurfaceNets::<C2>::sample(&self.samples, &local_voxel_positions);
      let case = SurfaceNets::<C2>::case(&values);
      let global_voxel_positions = SurfaceNets::<C2>::global_voxel_positions(UVec3::zero(), STEP, &local_voxel_positions);

      ui.grid("Cell Data Grid", |ui| {
        ui.label("Case");
        ui.monospace(format!("{}", case.0));
        ui.end_row();
        ui.label("Cell index");
        ui.monospace(format!("{}", self.selected_cell.to_index::<C2>()));
        ui.end_row();
      });

      ui.collapsing_open_with_grid("Voxels", "Voxels Grid", |ui| {
        ui.label("#");
        ui.label("voxel idx");
        ui.label("local");
        ui.label("global");
        ui.label("value");
        ui.end_row();
        for i in 0..8 {
          let local = local_voxel_positions[i];
          ui.monospace(format!("{}", i));
          ui.monospace(format!("{}", local.display()));
          ui.monospace(format!("{}", global_voxel_positions[i].display()));
          let value = self.samples.sample_mut(local);
          let response = ui.drag("", value, 0.01);
          if response.secondary_clicked() {
            *value *= -1.0;
          }
          if response.middle_clicked() {
            *value = 0.0;
          }
          ui.end_row();
        }
      });
    });
  }


  pub fn extract_chunk(&self, surface_nets: &SurfaceNets<C2>, chunk_vertices: &mut ChunkMesh) {
    surface_nets.extract_chunk(MIN, STEP, &ChunkSamples::Mixed(self.samples), chunk_vertices);
  }

  pub fn debug_draw(&self, debug_renderer: &mut DebugRenderer) {
    // Voxels
    for z in 0..C2::VOXELS_IN_CHUNK_ROW {
      for y in 0..C2::VOXELS_IN_CHUNK_ROW {
        for x in 0..C2::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          let sample = self.samples.sample(position);
          if sample.is_sign_negative() {
            debug_renderer.draw_point(position.into(), Vec4::one(), 20.0);
          }
        }
      }
    }
    // Cells
    for z in 0..C2::CELLS_IN_CHUNK_ROW {
      for y in 0..C2::CELLS_IN_CHUNK_ROW {
        for x in 0..C2::CELLS_IN_CHUNK_ROW {
          let position = MIN + UVec3::new(x, y, z);
          debug_renderer.draw_cube_lines(position.into(), 1.0, Vec4::one());
        }
      }
    }
  }
}
