use job_queue::{DepKey, In};

use crate::chunk::mesh::Vertex;
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABBSized;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::octmap::{LodJob, LodJobOutput};
use crate::lod::render::LodDraw;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize>: Clone + Send + Sync + 'static {
  type Chunk: LodChunkMesh + Send + Sync + 'static;
  type JobInput: In;
  type DependencyKey: DepKey;
  type DependenciesIterator<V: Volume>: Iterator<Item=(Self::DependencyKey, LodJob<C, V, Self>)> + ExactSizeIterator + Send + 'static;

  fn create_job<V: Volume>(
    &self,
    root_size: u32,
    aabb: AABBSized,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>);

  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)],
  ) -> Self::Chunk;

  fn update_render_data(
    &self,
    chunk: &Self::Chunk,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<LodDraw>,
  );
}
