use ultraviolet::{Isometry3, Vec3};

use crate::chunk::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::extract::LodExtractor;

/// LOD chunk mesh.
pub trait LodChunkMesh: Default + Send {
  fn is_empty(&self) -> bool;

  fn clear(&mut self);
}

/// Transforms a volume into chunk meshes while taking into account level of detail (LOD).
pub trait LodChunkMeshManager<C: ChunkSize>: LodChunkMeshManagerParameters {
  type Extractor: LodExtractor<C>;

  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(<<Self as LodChunkMeshManager<C>>::Extractor as LodExtractor<C>>::Chunk, bool))> + '_>);

  fn get_extractor(&self) -> &Self::Extractor;
}

/// Parameters for transformation, in a separate trait as they do not depend on the kind of chunks.
pub trait LodChunkMeshManagerParameters {
  fn get_max_lod_level(&self) -> u32;

  fn get_lod_factor(&self) -> f32;
  fn get_lod_factor_mut(&mut self) -> &mut f32;
}

// Box forwarders

impl<C: ChunkSize, E: LodExtractor<C>, T> LodChunkMeshManager<C> for Box<T> where
  T: LodChunkMeshManager<C, Extractor=E> + ?Sized
{
  type Extractor = E;
  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(E::Chunk, bool))> + '_>) { (**self).update(position) }
  #[inline]
  fn get_extractor(&self) -> &E { (**self).get_extractor() }
}

impl<T: LodChunkMeshManagerParameters + ?Sized> LodChunkMeshManagerParameters for Box<T> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { (**self).get_max_lod_level() }
  #[inline]
  fn get_lod_factor(&self) -> f32 { (**self).get_lod_factor() }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { (**self).get_lod_factor_mut() }
}
