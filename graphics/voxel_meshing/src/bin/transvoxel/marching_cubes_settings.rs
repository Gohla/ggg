use egui::{Align2, ComboBox};
use ultraviolet::UVec3;

use app::GuiFrame;
use gfx::display_math::UVec3DisplayExt;
use gui_widget::UiWidgetsExt;
use voxel_meshing::chunk::ChunkSampleArray;
use voxel_meshing::marching_cubes;
use voxel_meshing::marching_cubes::RegularCell;

use crate::{C1, MC};

#[derive(Default)]
pub struct MarchingCubesSettings {
  gui_equivalence_class: u8,
}

impl MarchingCubesSettings {
  pub fn render_gui(&mut self, gui_frame: &GuiFrame, samples: &mut ChunkSampleArray<C1>) {
    egui::Window::new("Marching Cubes")
      .anchor(Align2::LEFT_TOP, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        ui.collapsing_open("Cell", |ui| {
          ui.horizontal(|ui| {
            ComboBox::from_id_source("Equivalence class")
              .selected_text(format!("{:?}", self.gui_equivalence_class))
              .show_ui(ui, |ui| {
                for i in 0..18 {
                  ui.selectable_value(&mut self.gui_equivalence_class, i, format!("{:?}", i));
                }
              });
            if ui.button("Set").clicked() {
              let inside = -1.0;
              match self.gui_equivalence_class {
                0 => {
                  samples.set_all_to(1.0);
                }
                1 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                }
                2 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                3 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                }
                4 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 0, inside);
                }
                5 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                6 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                7 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                8 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                9 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(1, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                10 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                11 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                12 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                }
                13 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                14 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                15 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                16 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                }
                17 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                _ => {}
              }
            }
          });
          ui.horizontal(|ui| {
            if ui.button("Flip").clicked() {
              samples.flip_all();
            }
            if ui.button("To +0.0").clicked() {
              samples.set_all_to(0.0);
            }
            if ui.button("To -0.0").clicked() {
              samples.set_all_to(-0.0);
            }
            if ui.button("To +1.0").clicked() {
              samples.set_all_to(1.0);
            }
            if ui.button("To -1.0").clicked() {
              samples.set_all_to(-1.0);
            }
            ui.end_row();
          });
        });

        let local_coordinates = MC::local_coordinates(RegularCell::new(0, 0, 0));
        let global_coordinates = MC::global_coordinates(UVec3::zero(), 1, &local_coordinates);
        let values = MC::sample(samples, &local_coordinates);
        let case = MC::case(&values);
        let cell_class = marching_cubes::tables::REGULAR_CELL_CLASS[case as usize] as usize;
        let triangulation_info = marching_cubes::tables::REGULAR_CELL_DATA[cell_class];
        let vertex_count = triangulation_info.get_vertex_count() as usize;
        let triangle_count = triangulation_info.get_triangle_count() as usize;
        let vertices_data = marching_cubes::tables::REGULAR_VERTEX_DATA[case as usize];

        ui.collapsing_open_with_grid("Data", "Data Grid", |ui| {
          ui.label("Case");
          ui.monospace(format!("{case} (class: {cell_class})"));
          ui.end_row();
          ui.label("Counts");
          ui.monospace(format!("Vertices: {vertex_count}, triangles: {triangle_count}"));
          ui.end_row();
        });

        ui.collapsing_open_with_grid("Voxels", "Voxels Grid", |ui| {
          ui.label("#");
          ui.label("local");
          ui.label("global");
          ui.label("value");
          ui.end_row();
          for i in 0..8 {
            let local = local_coordinates[i];
            ui.monospace(format!("{}", i));
            ui.monospace(format!("{}", local.display()));
            ui.monospace(format!("{}", global_coordinates[i].display()));
            let value = samples.sample_mut(local);
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

        ui.collapsing_open_with_grid("Vertices", "Vertices", |ui| {
          ui.label("#");
          ui.label("-x?");
          ui.label("-y?");
          ui.label("-z?");
          ui.label("new?");
          ui.label("vtx idx");
          ui.label("vox a idx");
          ui.label("vox b idx");
          ui.end_row();
          for (i, vd) in vertices_data[0..vertex_count].iter().enumerate() {
            ui.monospace(format!("{}", i));
            ui.monospace(format!("{}", vd.subtract_x()));
            ui.monospace(format!("{}", vd.subtract_y()));
            ui.monospace(format!("{}", vd.subtract_z()));
            ui.monospace(format!("{}", vd.new_vertex()));
            ui.monospace(format!("{}", vd.vertex_index()));
            ui.monospace(format!("{}", vd.voxel_a_index()));
            ui.monospace(format!("{}", vd.voxel_b_index()));
            ui.end_row();
          }
        });

        ui.collapsing_open_with_grid("Triangulation", "Triangulation Grid", |ui| {
          ui.label("#");
          ui.label("triangle idxs");
          ui.end_row();
          for i in (0..triangle_count * 3).step_by(3) {
            ui.monospace(format!("{}", i / 3));
            ui.monospace(format!("{} {} {}", triangulation_info.vertex_index[i + 0], triangulation_info.vertex_index[i + 1], triangulation_info.vertex_index[i + 2]));
            ui.end_row();
          }
        });
      });
  }
}
