use egui::{Align2, Ui};
use ultraviolet::{UVec3, Vec4};

use app::GuiFrame;
use gfx::debug_renderer::DebugRenderer;
use gui_widget::UiWidgetsExt;
use voxel::chunk::{ChunkSampleArray, ChunkSamples, ChunkSize, ChunkVertices};
use voxel::surface_nets::SurfaceNets;

use crate::C;

pub type SN = SurfaceNets<C>;

pub const MIN: UVec3 = UVec3::new(0, 0, 0);
pub const STEP: u32 = 1;

#[derive(Default)]
pub struct SurfaceNetsDebugging {
  surface_nets: SN,
  samples: ChunkSampleArray<C>,
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
    // self.draw_data_gui(ui);
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
      for z in 0..C::VOXELS_IN_CHUNK_ROW {
        ui.collapsing_open_with_grid(format!("Z={}", z), format!("Grid Z={}", z), |ui| {
          ui.label("");
          for x in 0..C::VOXELS_IN_CHUNK_ROW {
            ui.label(format!("{}", x));
          }
          ui.end_row();
          for y in 0..C::VOXELS_IN_CHUNK_ROW {
            ui.label(format!("Y={}", y));
            for x in 0..C::VOXELS_IN_CHUNK_ROW {
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

  // fn draw_data_gui(&mut self, ui: &mut Ui) {
  //   let local_coordinates = SN::local_coordinates(RegularCell::new(0, 0, 0));
  //   let global_coordinates = SN::global_coordinates(UVec3::zero(), STEP, &local_coordinates);
  //   let values = SN::sample(&self.samples, &local_coordinates);
  //   let case = SN::case(&values);
  //   let cell_class = marching_cubes::tables::REGULAR_CELL_CLASS[case as usize] as usize;
  //   let triangulation_info = marching_cubes::tables::REGULAR_CELL_DATA[cell_class];
  //   let vertex_count = triangulation_info.get_vertex_count() as usize;
  //   let triangle_count = triangulation_info.get_triangle_count() as usize;
  //   let vertices_data = marching_cubes::tables::REGULAR_VERTEX_DATA[case as usize];
  //
  //   ui.collapsing_open_with_grid("Data", "Data Grid", |ui| {
  //     ui.label("Case");
  //     ui.monospace(format!("{case} (class: {cell_class})"));
  //     ui.end_row();
  //     ui.label("Counts");
  //     ui.monospace(format!("Vertices: {vertex_count}, triangles: {triangle_count}"));
  //     ui.end_row();
  //   });
  //
  //   ui.collapsing_open_with_grid("Voxels", "Voxels Grid", |ui| {
  //     ui.label("#");
  //     ui.label("local");
  //     ui.label("global");
  //     ui.label("value");
  //     ui.end_row();
  //     for i in 0..8 {
  //       let local = local_coordinates[i];
  //       ui.monospace(format!("{}", i));
  //       ui.monospace(format!("{}", local.display()));
  //       ui.monospace(format!("{}", global_coordinates[i].display()));
  //       let value = self.samples.sample_mut(local);
  //       let response = ui.drag("", value, 0.01);
  //       if response.secondary_clicked() {
  //         *value *= -1.0;
  //       }
  //       if response.middle_clicked() {
  //         *value = 0.0;
  //       }
  //       ui.end_row();
  //     }
  //   });
  //
  //   ui.collapsing_open_with_grid("Vertices", "Vertices", |ui| {
  //     ui.label("#");
  //     ui.label("-x?");
  //     ui.label("-y?");
  //     ui.label("-z?");
  //     ui.label("new?");
  //     ui.label("vtx idx");
  //     ui.label("vox a idx");
  //     ui.label("vox b idx");
  //     ui.end_row();
  //     for (i, vd) in vertices_data[0..vertex_count].iter().enumerate() {
  //       ui.monospace(format!("{}", i));
  //       ui.monospace(format!("{}", vd.subtract_u()));
  //       ui.monospace(format!("{}", vd.subtract_v()));
  //       ui.monospace(format!("{}", vd.subtract_w()));
  //       ui.monospace(format!("{}", vd.new_vertex()));
  //       ui.monospace(format!("{}", vd.vertex_index()));
  //       ui.monospace(format!("{}", vd.voxel_a_index()));
  //       ui.monospace(format!("{}", vd.voxel_b_index()));
  //       ui.end_row();
  //     }
  //   });
  //
  //   ui.collapsing_open_with_grid("Triangulation", "Triangulation Grid", |ui| {
  //     ui.label("#");
  //     ui.label("triangle idxs");
  //     ui.end_row();
  //     for i in (0..triangle_count * 3).step_by(3) {
  //       ui.monospace(format!("{}", i / 3));
  //       ui.monospace(format!("{} {} {}", triangulation_info.vertex_index[i + 0], triangulation_info.vertex_index[i + 1], triangulation_info.vertex_index[i + 2]));
  //       ui.end_row();
  //     }
  //   });
  // }

  pub fn extract_chunk(&self, chunk_vertices: &mut ChunkVertices) {
    self.surface_nets.extract_chunk(MIN, STEP, &ChunkSamples::Mixed(self.samples), chunk_vertices);
  }

  pub fn debug_draw(&self, debug_renderer: &mut DebugRenderer) {
    // Voxels
    for z in 0..C::VOXELS_IN_CHUNK_ROW {
      for y in 0..C::VOXELS_IN_CHUNK_ROW {
        for x in 0..C::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          let sample = self.samples.sample(position);
          if sample.is_sign_negative() {
            debug_renderer.draw_point(position.into(), Vec4::one(), 20.0);
          }
        }
      }
    }
    // Cells
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        for x in 0..C::CELLS_IN_CHUNK_ROW {
          let position = MIN + UVec3::new(x, y, z);
          debug_renderer.draw_cube_lines(position.into(), 1.0, Vec4::one());
        }
      }
    }
  }
}
