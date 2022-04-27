use ultraviolet::{Isometry3, Vec3};

use crate::lod::aabb::AABB;

/// LOD chunk vertices
pub trait LodChunkVertices: Default + Send {
  fn is_empty(&self) -> bool;

  fn clear(&mut self);
}

/// Transforms a volume into chunk vertices while taking into account level of detail (LOD)
pub trait LodChunkVerticesManager<C: LodChunkVertices>: LodChunkVerticesManagerParameters {
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(C, bool))> + '_>);
}

/// Parameters for transformation, in a separate trait as they do not depend on the kind of chunks.
pub trait LodChunkVerticesManagerParameters {
  fn get_max_lod_level(&self) -> u32;
  fn get_lod_factor(&self) -> f32;
  fn get_lod_factor_mut(&mut self) -> &mut f32;
}

// Box forwarders

impl<C: LodChunkVertices, T: LodChunkVerticesManager<C> + ?Sized> LodChunkVerticesManager<C> for Box<T> {
  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(C, bool))> + '_>) { (**self).update(position) }
}

impl<T: LodChunkVerticesManagerParameters + ?Sized> LodChunkVerticesManagerParameters for Box<T> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { (**self).get_max_lod_level() }
  #[inline]
  fn get_lod_factor(&self) -> f32 { (**self).get_lod_factor() }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { (**self).get_lod_factor_mut() }
}
