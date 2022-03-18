use egui::Align2;
use ultraviolet::UVec3;

use app::GuiFrame;
use voxel_meshing::chunk::{ChunkSamples, ChunkVertices};
use voxel_meshing::transvoxel::side::TransitionSide;
use voxel_meshing::transvoxel::Transvoxel;

use crate::C1;
use crate::marching_cubes_debugging::LORES_STEP;

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

  pub fn extract_chunk(
    &self,
    side: TransitionSide,
    hires_chunk_mins: &[UVec3; 4],
    hires_chunk_samples: &[ChunkSamples<C1>; 4],
    chunk_vertices: &mut ChunkVertices,
  ) {
    self.transvoxel.extract_chunk(side, hires_chunk_mins, hires_chunk_samples, HIRES_STEP, UVec3::zero(), LORES_STEP, chunk_vertices);
  }
}
