use crate::chunk::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_vertices::LodChunkVertices;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize>: Clone + Send + 'static {
  type Chunk: LodChunkVertices;

  fn extract<V: Volume>(&self, total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk);
}

// Box forwarders

impl<C: ChunkSize, CV: LodChunkVertices, T: LodExtractor<C, Chunk=CV> + ?Sized> LodExtractor<C> for Box<T> {
  type Chunk = CV;

  #[inline]
  fn extract<V: Volume>(&self, total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    (**self).extract(total_size, aabb, volume, chunk)
  }
}
