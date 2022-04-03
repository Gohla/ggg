use ultraviolet::{Isometry3, Vec3};

use crate::chunk::ChunkVertices;
use crate::lod::aabb::AABB;

// Trait

pub trait LodChunkManager {
  fn get_max_lod_level(&self) -> u32;
  fn get_lod_factor(&self) -> f32;
  fn get_lod_factor_mut(&mut self) -> &mut f32;

  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(LodChunkVertices, bool))> + '_>);
}

// Box wrapper

impl<T: LodChunkManager + ?Sized> LodChunkManager for Box<T> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { (**self).get_max_lod_level() }
  #[inline]
  fn get_lod_factor(&self) -> f32 { (**self).get_lod_factor() }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { (**self).get_lod_factor_mut() }
  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(LodChunkVertices, bool))> + '_>) { (**self).update(position) }
}

// LOD chunk vertices

#[derive(Clone, Default, Debug)]
pub struct LodChunkVertices {
  pub regular: ChunkVertices,
  pub transition_lo_x_chunk: ChunkVertices,
  pub transition_hi_x_chunk: ChunkVertices,
  pub transition_lo_y_chunk: ChunkVertices,
  pub transition_hi_y_chunk: ChunkVertices,
  pub transition_lo_z_chunk: ChunkVertices,
  pub transition_hi_z_chunk: ChunkVertices,
}

impl LodChunkVertices {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(
    regular: ChunkVertices,
    transition_lo_x_chunk: ChunkVertices,
    transition_hi_x_chunk: ChunkVertices,
    transition_lo_y_chunk: ChunkVertices,
    transition_hi_y_chunk: ChunkVertices,
    transition_lo_z_chunk: ChunkVertices,
    transition_hi_z_chunk: ChunkVertices,
  ) -> Self {
    Self {
      regular,
      transition_lo_x_chunk,
      transition_hi_x_chunk,
      transition_lo_y_chunk,
      transition_hi_y_chunk,
      transition_lo_z_chunk,
      transition_hi_z_chunk,
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.regular.clear();
    self.transition_lo_x_chunk.clear();
    self.transition_hi_x_chunk.clear();
    self.transition_lo_y_chunk.clear();
    self.transition_hi_y_chunk.clear();
    self.transition_lo_z_chunk.clear();
    self.transition_hi_z_chunk.clear();
  }
}
