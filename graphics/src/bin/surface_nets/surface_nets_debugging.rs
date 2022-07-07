use std::fmt::{Display, Formatter};

use egui::{Align2, ComboBox, Ui};
use serde::{Deserialize, Serialize};
use ultraviolet::{UVec3, Vec4};

use app::GuiFrame;
use gfx::debug_renderer::DebugRenderer;
use gfx::display_math::{UVec3DisplayExt, Vec3DisplayExt};
use gui_widget::UiWidgetsExt;
use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::sample::{ChunkSampleArray, ChunkSamples, ChunkSampleSlice, ChunkSampleSliceMut, ChunkSamplesMut};
use voxel::chunk::shape::Shape;
use voxel::chunk::size::ChunkSize;
use voxel::surface_nets::{Cell, SurfaceNets};

use crate::{C2, C6};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SurfaceNetsDebugging {
  main_lores: Chunk,
  x_positive: Chunk,
  x_positive_y: Chunk,
  x_positive_z: Chunk,
  x_positive_yz: Chunk,

  chunk_sample_array: ChunkSampleArray<C6>,
  selected_chunk: SelectedChunk,
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
enum SelectedChunk {
  #[default]
  Main,
  XPositive,
  XPositiveY,
  XPositiveZ,
  XPositiveYZ,
}

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
struct Chunk {
  selected_cell: Cell,
}

impl SurfaceNetsDebugging {
  pub fn show_gui_window(&mut self, gui_frame: &GuiFrame) {
    egui::Window::new("Surface Nets")
      .anchor(Align2::LEFT_TOP, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        ComboBox::from_id_source("Selected Chunk")
          .selected_text(format!("{}", self.selected_chunk))
          .show_ui(ui, |ui| {
            for chunk in SelectedChunk::iter() {
              ui.selectable_value(&mut self.selected_chunk, chunk, format!("{}", chunk));
            }
          });
        self.draw_window_contents(ui);
      });
  }

  pub fn extract_chunk_and_debug_draw(&self, chunk_vertices: &mut ChunkMesh, debug_renderer: &mut DebugRenderer) {
    let surface_nets = SurfaceNets::<C2>::new();
    let r = C2::CELLS_IN_CHUNK_ROW;
    let main_lores_min_index = <C6 as ChunkSize>::VoxelChunkShape::index_from_xyz(0, 0, 0);
    let main_lores_max_index = <C6 as ChunkSize>::VoxelChunkShape::index_from_xyz(r, r, r);
    self.main_lores.extract_chunk_and_debug_draw(2, UVec3::zero(), surface_nets, &self.chunk_sample_array.slice::<C2, _>(main_lores_min_index..=main_lores_max_index), chunk_vertices, debug_renderer);
    let x = r * 2;
    let x_positive_min = UVec3::new(x, 0, 0);
    self.x_positive.extract_chunk_and_debug_draw(1, UVec3::new(x, 0, 0), surface_nets, &self.chunk_sample_array, chunk_vertices, debug_renderer);
    self.x_positive_y.extract_chunk_and_debug_draw(1, UVec3::new(x, r, 0), surface_nets, &self.chunk_sample_array, chunk_vertices, debug_renderer);
    self.x_positive_z.extract_chunk_and_debug_draw(1, UVec3::new(x, 0, r), surface_nets, &self.chunk_sample_array, chunk_vertices, debug_renderer);
    self.x_positive_yz.extract_chunk_and_debug_draw(1, UVec3::new(x, r, r), surface_nets, &self.chunk_sample_array, chunk_vertices, debug_renderer);
    // Voxels
    for z in 0..C6::VOXELS_IN_CHUNK_ROW {
      for y in 0..C6::VOXELS_IN_CHUNK_ROW {
        for x in 0..C6::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          let sample = self.chunk_sample_array.sample(position);
          if sample.is_sign_negative() {
            debug_renderer.draw_point(position.into(), Vec4::one(), 20.0);
          }
        }
      }
    }
  }

  fn draw_window_contents(&mut self, ui: &mut Ui) {
    let o = C2::CELLS_IN_CHUNK_ROW;
    let x = o * 2;
    match self.selected_chunk {
      SelectedChunk::Main => self.main_lores.draw_window_contents(ui, 2, UVec3::zero(), &mut self.chunk_sample_array),
      SelectedChunk::XPositive => self.x_positive.draw_window_contents(ui, 1, UVec3::new(x, 0, 0), &mut self.chunk_sample_array),
      SelectedChunk::XPositiveY => self.x_positive_y.draw_window_contents(ui, 1, UVec3::new(x, o, 0), &mut self.chunk_sample_array),
      SelectedChunk::XPositiveZ => self.x_positive_z.draw_window_contents(ui, 1, UVec3::new(x, 0, o), &mut self.chunk_sample_array),
      SelectedChunk::XPositiveYZ => self.x_positive_yz.draw_window_contents(ui, 1, UVec3::new(x, o, o), &mut self.chunk_sample_array),
    }
  }
}

impl SelectedChunk {
  fn iter() -> impl IntoIterator<Item=Self> {
    use SelectedChunk::*;
    [Main, XPositive, XPositiveY, XPositiveZ, XPositiveYZ]
  }
}

impl Display for SelectedChunk {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      SelectedChunk::Main => f.write_str("Main"),
      SelectedChunk::XPositive => f.write_str("+X[]"),
      SelectedChunk::XPositiveY => f.write_str("+X[Y]"),
      SelectedChunk::XPositiveZ => f.write_str("+X[Z]"),
      SelectedChunk::XPositiveYZ => f.write_str("+X[YZ]"),
    }
  }
}

impl Chunk {
  fn draw_window_contents(&mut self, ui: &mut Ui, step: u32, minimum_point: UVec3, samples: &mut ChunkSampleSliceMut<C2>) {
    self.draw_cell_gui(ui, samples);
    self.draw_data_gui(ui, step, minimum_point, samples);
  }

  fn draw_cell_gui(&mut self, ui: &mut Ui, samples: &mut ChunkSampleSliceMut<C2>) {
    ui.collapsing_open("Cell", |ui| {
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
              let sample = samples.sample_mut(UVec3::new(x, y, z));
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

  fn draw_data_gui(&mut self, ui: &mut Ui, step: u32, minimum_point: UVec3, samples: &mut ChunkSampleSliceMut<C2>) {
    ui.collapsing_open("Cell Data", |ui| {
      ui.horizontal(|ui| {
        ui.label("Selected Cell");
        ui.drag_range("x: ", &mut self.selected_cell.x, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
        ui.drag_range("y: ", &mut self.selected_cell.y, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
        ui.drag_range("z: ", &mut self.selected_cell.z, 1, 0..=C2::CELLS_IN_CHUNK_ROW - 1);
      });

      let local_voxel_positions = SurfaceNets::<C2>::local_voxel_positions(self.selected_cell);
      let values = SurfaceNets::<C2>::sample(samples, &local_voxel_positions);
      let case = SurfaceNets::<C2>::case(&values);
      let global_voxel_positions = SurfaceNets::<C2>::global_voxel_positions(minimum_point, step, &local_voxel_positions);

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
        ui.label("local");
        ui.label("global");
        ui.label("value");
        ui.end_row();
        for i in 0..8 {
          let local = local_voxel_positions[i];
          ui.monospace(format!("{}", i));
          ui.monospace(format!("{}", local.display()));
          ui.monospace(format!("{}", global_voxel_positions[i].display()));
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
    });
  }

  fn extract_chunk_and_debug_draw(
    &self,
    step: u32,
    minimum_point: UVec3,
    surface_nets: SurfaceNets<C2>,
    chunk_samples: &ChunkSampleSlice<C2>,
    chunk_mesh: &mut ChunkMesh,
    debug_renderer: &mut DebugRenderer
  ) {
    surface_nets.extract_chunk_from_samples(minimum_point, step, chunk_samples, chunk_mesh);
    self.debug_draw(step, minimum_point, debug_renderer);
  }

  fn debug_draw(&self, step: u32, minimum_point: UVec3, debug_renderer: &mut DebugRenderer) {
    for z in 0..C2::CELLS_IN_CHUNK_ROW {
      for y in 0..C2::CELLS_IN_CHUNK_ROW {
        for x in 0..C2::CELLS_IN_CHUNK_ROW {
          let position = minimum_point + UVec3::new(x, y, z) * step;
          debug_renderer.draw_cube_lines(position.into(), step as f32, Vec4::one());
        }
      }
    }
  }
}
