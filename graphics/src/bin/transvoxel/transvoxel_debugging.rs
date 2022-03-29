use egui::Align2;

use app::GuiFrame;
use voxel::chunk::ChunkVertices;
use voxel::transvoxel::side::TransitionSide;
use voxel::transvoxel::Transvoxel;

use crate::C1;
use crate::chunk_manager::TvLoZChunkManager;
use crate::marching_cubes_debugging::{LORES_MIN, LORES_STEP};

pub type TV = Transvoxel<C1>;

pub const HIRES_STEP: u32 = 1;

#[derive(Default)]
pub struct TransvoxelDebugging {
  transvoxel: TV,
  _equivalence_class: u8,
}

impl TransvoxelDebugging {
  pub fn render_gui(&mut self, gui_frame: &GuiFrame) {
    egui::Window::new("Transvoxel")
      .anchor(Align2::RIGHT_TOP, egui::Vec2::default())
      .show(&gui_frame, |_ui| {});
  }

  pub fn extract_loz_chunk(
    &self,
    chunk_manager: TvLoZChunkManager,
    chunk_vertices: &mut ChunkVertices,
  ) {
    let (hires_chunk_mins, hires_chunk_samples) = chunk_manager.create_mins_and_samples();
    self.transvoxel.extract_chunk(TransitionSide::LoZ, &hires_chunk_mins, &hires_chunk_samples, HIRES_STEP, LORES_MIN, LORES_STEP, chunk_vertices);
  }
}
