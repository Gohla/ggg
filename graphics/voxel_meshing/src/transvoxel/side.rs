use ultraviolet::{UVec3, Vec3};
use crate::chunk::Chunk;

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
        let min = aabb.min();
        let cen = aabb.center();
        let x = min.x + aabb.size();
        [
          UVec3::new(x, min.y, min.z),
          UVec3::new(x, min.y, cen.z),
          UVec3::new(x, cen.y, min.z),
          UVec3::new(x, cen.y, cen.z),
        ]
      }
      TransitionSide::LoY => {
        let min = aabb.min();
        let cen = aabb.center();
        let extends = aabb.extends();
        let y = min.y - extends;
        [
          UVec3::new(min.x, y, min.z),
          UVec3::new(cen.x, y, min.z),
          UVec3::new(min.x, y, cen.z),
          UVec3::new(cen.x, y, cen.z),
        ]
      }
      TransitionSide::HiY => {
        let min = aabb.min();
        let cen = aabb.center();
        let y = min.y + aabb.size();
        [
          UVec3::new(min.x, y, min.z),
          UVec3::new(cen.x, y, min.z),
          UVec3::new(min.x, y, cen.z),
          UVec3::new(cen.x, y, cen.z),
        ]
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
        let min = aabb.min();
        let cen = aabb.center();
        let z = min.z + aabb.size();
        [
          UVec3::new(min.x, min.y, z),
          UVec3::new(cen.x, min.y, z),
          UVec3::new(min.x, cen.y, z),
          UVec3::new(cen.x, cen.y, z),
        ]
      }
    }
  }


  #[inline]
  pub fn get_hires_local_voxels<C: Chunk>(&self, u: u32, v: u32) -> [UVec3; 9] {
    let u = (u % C::HALF_CELLS_IN_CHUNK_ROW) * 2;
    let v = (v % C::HALF_CELLS_IN_CHUNK_ROW) * 2;
    match self {
      TransitionSide::LoX => {
        Self::add_hires_voxels(UVec3::new(C::CELLS_IN_CHUNK_ROW, v, u), &Self::X_HIRES_VOXELS)
      }
      TransitionSide::HiX => {
        Self::add_hires_voxels(UVec3::new(0, v, u), &Self::X_HIRES_VOXELS)
      }
      TransitionSide::LoY => {
        Self::add_hires_voxels(UVec3::new(u, C::CELLS_IN_CHUNK_ROW, v), &Self::Y_HIRES_VOXELS)
      }
      TransitionSide::HiY => {
        Self::add_hires_voxels(UVec3::new(u, 0, v), &Self::Y_HIRES_VOXELS)
      }
      TransitionSide::LoZ => {
        Self::add_hires_voxels(UVec3::new(u, v, C::CELLS_IN_CHUNK_ROW), &Self::Z_HIRES_VOXELS)
      }
      TransitionSide::HiZ => {
        Self::add_hires_voxels(UVec3::new(u, v, 0), &Self::Z_HIRES_VOXELS)
      }
    }
  }

  const X_HIRES_VOXELS: [UVec3; 9] = [
    UVec3::new(0, 0, 0), // 0 & 9
    UVec3::new(0, 0, 1), // 1
    UVec3::new(0, 0, 2), // 2 & A
    UVec3::new(0, 1, 0), // 3
    UVec3::new(0, 1, 1), // 4
    UVec3::new(0, 1, 2), // 5
    UVec3::new(0, 2, 0), // 6 & B
    UVec3::new(0, 2, 1), // 7
    UVec3::new(0, 2, 2), // 8 & C
  ];
  const Y_HIRES_VOXELS: [UVec3; 9] = [
    UVec3::new(0, 0, 0), // 0 & 9
    UVec3::new(1, 0, 0), // 1
    UVec3::new(2, 0, 0), // 2 & A
    UVec3::new(0, 0, 1), // 3
    UVec3::new(1, 0, 1), // 4
    UVec3::new(2, 0, 1), // 5
    UVec3::new(0, 0, 2), // 6 & B
    UVec3::new(1, 0, 2), // 7
    UVec3::new(2, 0, 2), // 8 & C
  ];
  const Z_HIRES_VOXELS: [UVec3; 9] = [
    UVec3::new(0, 0, 0), // 0 & 9
    UVec3::new(1, 0, 0), // 1
    UVec3::new(2, 0, 0), // 2 & A
    UVec3::new(0, 1, 0), // 3
    UVec3::new(1, 1, 0), // 4
    UVec3::new(2, 1, 0), // 5
    UVec3::new(0, 2, 0), // 6 & B
    UVec3::new(1, 2, 0), // 7
    UVec3::new(2, 2, 0), // 8 & C
  ];

  #[inline]
  fn add_hires_voxels(base: UVec3, voxels: &[UVec3; 9]) -> [UVec3; 9] {
    [
      base + voxels[0],
      base + voxels[1],
      base + voxels[2],
      base + voxels[3],
      base + voxels[4],
      base + voxels[5],
      base + voxels[6],
      base + voxels[7],
      base + voxels[8],
    ]
  }


  #[inline]
  pub fn get_lores_local_voxels<C: Chunk>(&self, u: u32, v: u32) -> [Vec3; 4] {
    let u = u as f32;
    let v = v as f32;
    let transition_width = 1.0; // TODO: determine width of transition cell consistently and based on LOD.
    match self {
      TransitionSide::LoX => {
        Self::add_lores_voxels(Vec3::new(transition_width, v, u), &Self::X_LORES_VOXELS)
      }
      TransitionSide::HiX => {
        Self::add_lores_voxels(Vec3::new(C::CELLS_IN_CHUNK_ROW_F32 - transition_width, v, u), &Self::X_LORES_VOXELS)
      }
      TransitionSide::LoY => {
        Self::add_lores_voxels(Vec3::new(u, transition_width, v), &Self::Y_LORES_VOXELS)
      }
      TransitionSide::HiY => {
        Self::add_lores_voxels(Vec3::new(u, C::CELLS_IN_CHUNK_ROW_F32 - transition_width, v), &Self::Y_LORES_VOXELS)
      }
      TransitionSide::LoZ => {
        Self::add_lores_voxels(Vec3::new(u, v, transition_width), &Self::Z_LORES_VOXELS)
      }
      TransitionSide::HiZ => {
        Self::add_lores_voxels(Vec3::new(u, v, C::CELLS_IN_CHUNK_ROW_F32 - transition_width), &Self::Z_LORES_VOXELS)
      }
    }
  }

  const X_LORES_VOXELS: [Vec3; 4] = [
    Vec3::new(0.0, 0.0, 0.0), // 9
    Vec3::new(0.0, 0.0, 1.0), // A
    Vec3::new(0.0, 1.0, 0.0), // B
    Vec3::new(0.0, 1.0, 1.0), // C
  ];
  const Y_LORES_VOXELS: [Vec3; 4] = [
    Vec3::new(0.0, 0.0, 0.0), // 9
    Vec3::new(1.0, 0.0, 0.0), // A
    Vec3::new(0.0, 0.0, 1.0), // B
    Vec3::new(1.0, 0.0, 1.0), // C
  ];
  const Z_LORES_VOXELS: [Vec3; 4] = [
    Vec3::new(0.0, 0.0, 0.0), // 9
    Vec3::new(1.0, 0.0, 0.0), // A
    Vec3::new(0.0, 1.0, 0.0), // B
    Vec3::new(1.0, 1.0, 0.0), // C
  ];


  #[inline]
  fn add_lores_voxels(base: Vec3, voxels: &[Vec3; 4]) -> [Vec3; 4] {
    [
      base + voxels[0],
      base + voxels[1],
      base + voxels[2],
      base + voxels[3],
    ]
  }
}
