use ultraviolet::{UVec2, UVec3, Vec3};

use crate::chunk::CELLS_IN_CHUNK_ROW;
use crate::octree::AABB;

flagset::flags! {
   pub enum TransitionSide: u8 {
        LoX,
        HiX,
        LoY,
        HiY,
        LoZ,
        HiZ,
    }
}
pub type TransitionSides = flagset::FlagSet<TransitionSide>;

impl TransitionSide {
  #[inline]
  pub fn subdivided_face_of_side_minimums(&self, aabb: AABB) -> [UVec3; 4] {
    match self {
      TransitionSide::LoX => {
        let min = aabb.min();
        let cen = aabb.center();
        let extends = aabb.extends();
        let x = min.x - extends;
        [
          UVec3::new(x, min.y, min.z),
          UVec3::new(x, min.y, cen.z),
          UVec3::new(x, cen.y, min.z),
          UVec3::new(x, cen.y, cen.z),
        ]
      }
      TransitionSide::HiX => {
        todo!()
      }
      TransitionSide::LoY => {
        todo!()
      }
      TransitionSide::HiY => {
        todo!()
      }
      TransitionSide::LoZ => {
        let min = aabb.min();
        let cen = aabb.center();
        let extends = aabb.extends();
        let z = min.z - extends;
        [
          UVec3::new(min.x, min.y, z),
          UVec3::new(cen.x, min.y, z),
          UVec3::new(min.x, cen.y, z),
          UVec3::new(cen.x, cen.y, z),
        ]
      }
      TransitionSide::HiZ => {
        todo!()
      }
    }
  }

  #[inline]
  pub fn get_hires_local_voxels(&self, cell: UVec2) -> [UVec3; 9] {
    let x = (cell.x % 8) * 2;
    let y = (cell.y % 8) * 2;
    match self {
      TransitionSide::LoX => {
        let cell_3d = UVec3::new(CELLS_IN_CHUNK_ROW, y, x);
        [
          cell_3d + UVec3::new(0, 0, 0), // 0 & 9
          cell_3d + UVec3::new(0, 0, 1), // 1
          cell_3d + UVec3::new(0, 0, 2), // 2 & A
          cell_3d + UVec3::new(0, 1, 0), // 3
          cell_3d + UVec3::new(0, 1, 1), // 4
          cell_3d + UVec3::new(0, 1, 2), // 5
          cell_3d + UVec3::new(0, 2, 0), // 6 & B
          cell_3d + UVec3::new(0, 2, 1), // 7
          cell_3d + UVec3::new(0, 2, 2), // 8 & C
        ]
      }
      TransitionSide::HiX => {
        todo!()
      }
      TransitionSide::LoY => {
        todo!()
      }
      TransitionSide::HiY => {
        todo!()
      }
      TransitionSide::LoZ => {
        let cell_3d = UVec3::new(x, y, CELLS_IN_CHUNK_ROW);
        [
          cell_3d + UVec3::new(0, 0, 0), // 0 & 9
          cell_3d + UVec3::new(1, 0, 0), // 1
          cell_3d + UVec3::new(2, 0, 0), // 2 & A
          cell_3d + UVec3::new(0, 1, 0), // 3
          cell_3d + UVec3::new(1, 1, 0), // 4
          cell_3d + UVec3::new(2, 1, 0), // 5
          cell_3d + UVec3::new(0, 2, 0), // 6 & B
          cell_3d + UVec3::new(1, 2, 0), // 7
          cell_3d + UVec3::new(2, 2, 0), // 8 & C
        ]
      }
      TransitionSide::HiZ => {
        todo!()
      }
    }
  }

  #[inline]
  pub fn get_lores_local_voxels(&self, cell: UVec2) -> [Vec3; 4] {
    let transition_width = 0.5; // TODO: determine width of transition cell consistently and based on LOD.
    match self {
      TransitionSide::LoX => {
        let cell_3d = Vec3::new(transition_width, cell.y as f32, cell.x as f32);
        [
          cell_3d + Vec3::new(0.0, 0.0, 0.0), // 9
          cell_3d + Vec3::new(0.0, 0.0, 1.0), // A
          cell_3d + Vec3::new(0.0, 1.0, 0.0), // B
          cell_3d + Vec3::new(0.0, 1.0, 1.0), // C
        ]
      }
      TransitionSide::HiX => {
        todo!()
      }
      TransitionSide::LoY => {
        todo!()
      }
      TransitionSide::HiY => {
        todo!()
      }
      TransitionSide::LoZ => {
        let cell_3d = Vec3::new(cell.x as f32, cell.y as f32, transition_width);
        [
          cell_3d + Vec3::new(0.0, 0.0, 0.0), // 9
          cell_3d + Vec3::new(1.0, 0.0, 0.0), // A
          cell_3d + Vec3::new(0.0, 1.0, 0.0), // B
          cell_3d + Vec3::new(1.0, 1.0, 0.0), // C
        ]
      }
      TransitionSide::HiZ => {
        todo!()
      }
    }
  }
}
