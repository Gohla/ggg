use egui::{Align2, ComboBox, Ui};
use ultraviolet::{UVec3, Vec4};

use gfx::debug_renderer::DebugRenderer;
use gfx::fmt_math::UVec3DisplayExt;
use gui::Gui;
use gui::widget::UiWidgetsExt;
use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::sample::{ChunkSampleArray, ChunkSamples, ChunkSamplesMut, MaybeCompressedChunkSampleArray};
use voxel::chunk::size::ChunkSize;
use voxel::marching_cubes;
use voxel::marching_cubes::{MarchingCubes, RegularCell};

use crate::C1;

pub type MC = MarchingCubes<C1>;

pub const MIN: UVec3 = UVec3::new(0, 0, 0);
pub const STEP: u32 = 1;

#[derive(Default)]
pub struct MarchingCubesDebugging {
  marching_cubes: MC,
  samples: ChunkSampleArray<C1>,
  equivalence_class: u8,
}

impl MarchingCubesDebugging {
  pub fn show(&mut self, gui: &Gui) {
    egui::Window::new("Marching Cubes")
      .constrain_to(gui.area_under_title_bar)
      .anchor(Align2::LEFT_TOP, egui::Vec2::default())
      .show(&gui, |ui| {
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
        ComboBox::from_id_source("Equivalence class")
          .selected_text(format!("{:?}", self.equivalence_class))
          .show_ui(ui, |ui| {
            for i in 0..18 {
              ui.selectable_value(&mut self.equivalence_class, i, format!("{:?}", i));
            }
          });
        if ui.button("Set").clicked() {
          let inside = -1.0;
          match self.equivalence_class {
            0 => {
              self.samples.set_all_to(1.0);
            }
            1 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 1, inside);
            }
            2 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            3 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
            }
            4 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 0, inside);
            }
            5 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
            }
            6 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            7 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            8 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
            }
            9 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 0, inside);
              self.samples.set(1, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            10 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            11 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 0, inside);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
            }
            12 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(0, 1, 1, inside);
            }
            13 => {
              self.samples.set_all_to(1.0);
              self.samples.set(0, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            14 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            15 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(0, 1, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            16 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(0, 1, 1, inside);
            }
            17 => {
              self.samples.set_all_to(1.0);
              self.samples.set(1, 0, 0, inside);
              self.samples.set(0, 1, 0, inside);
              self.samples.set(0, 0, 1, inside);
              self.samples.set(1, 0, 1, inside);
              self.samples.set(1, 1, 1, inside);
            }
            _ => {}
          }
        }
      });
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
    });
  }

  fn draw_data_gui(&mut self, ui: &mut Ui) {
    let local_coordinates = MC::local_coordinates(RegularCell::new(0, 0, 0));
    let global_coordinates = MC::global_coordinates(UVec3::zero(), STEP, &local_coordinates);
    let values = MC::sample(&self.samples, &local_coordinates);
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
        ui.monospace(format!("{}", vd.subtract_u()));
        ui.monospace(format!("{}", vd.subtract_v()));
        ui.monospace(format!("{}", vd.subtract_w()));
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
  }

  pub fn extract_chunk(&self, chunk_vertices: &mut ChunkMesh) {
    // HACK: pass LORES_STEP (2) here, to make global voxels draw as if this was a 2x2 chunk grid.
    self.marching_cubes.extract_chunk(MIN, STEP, &MaybeCompressedChunkSampleArray::Mixed(self.samples.clone()), chunk_vertices);
  }

  pub fn debug_draw(&self, debug_renderer: &mut DebugRenderer) {
    // Voxels
    for z in 0..C1::VOXELS_IN_CHUNK_ROW {
      for y in 0..C1::VOXELS_IN_CHUNK_ROW {
        for x in 0..C1::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          let sample = self.samples.sample(position);
          // HACK: multiply by LORES_STEP after sampling to draw as if this was a 2x2 chunk grid.
          let position = MIN + position * STEP;
          if sample.is_sign_negative() {
            debug_renderer.draw_point(position.into(), Vec4::one(), 20.0);
          }
        }
      }
    }
    // Cell
    debug_renderer.draw_cube_lines(MIN.into(), STEP as f32, Vec4::one());
  }
}
