use crate::chunk::{ChunkSize, Vertex};
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::render::LodDraw;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize>: Clone + Send + 'static {
  type Chunk: LodChunkMesh;

  fn extract<V: Volume>(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: &V,
    chunk: &mut Self::Chunk,
  );

  fn update_render_data(
    &self,
    chunk: &Self::Chunk,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<LodDraw>,
  );
}

// Box forwarders

impl<C: ChunkSize, CM: LodChunkMesh, T: LodExtractor<C, Chunk=CM> + ?Sized> LodExtractor<C> for Box<T> {
  type Chunk = CM;

  #[inline]
  fn extract<V: Volume>(&self, total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    (**self).extract::<V>(total_size, aabb, volume, chunk)
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    (**self).update_render_data(chunk, vertices, indices, draws);
  }
}
