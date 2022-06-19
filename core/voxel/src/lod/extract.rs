use job_queue::{DepKey, In};

use crate::chunk::mesh::Vertex;
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::octmap::{LodJob, LodJobOutput};
use crate::lod::render::LodDraw;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize, V: Volume>: Clone + Send + Sync + 'static {
  type Chunk: LodChunkMesh + Send + Sync + 'static;
  type JobInput: In;
  type DependencyKey: DepKey;
  type DependenciesIntoIterator: IntoIterator<Item=(Self::DependencyKey, LodJob<V, Self::JobInput, Self::DependenciesIntoIterator>)>;

  fn create_job(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIntoIterator);

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

// Box forwarders

impl<
  C: ChunkSize,
  V: Volume,
  CM: LodChunkMesh,
  JI: In,
  DK: DepKey,
  DI: IntoIterator<Item=(DK, LodJob<V, JI, DI>)>,
  T: LodExtractor<C, V, Chunk=CM, DependencyKey=DK, JobInput=JI, DependenciesIntoIterator=DI> + ?Sized
> LodExtractor<C, V> for Box<T> {
  type Chunk = CM;
  type JobInput = JI;
  type DependencyKey = DK;
  type DependenciesIntoIterator = DI;

  #[inline]
  fn create_job(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIntoIterator) {
    (**self).create_job(total_size, aabb, volume, empty_lod_chunk_mesh)
  }

  #[inline]
  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    (**self).run_job(input, dependency_outputs)
  }

  #[inline]
  fn update_render_data(
    &self,
    chunk: &Self::Chunk,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<LodDraw>,
  ) {
    (**self).update_render_data(chunk, vertices, indices, draws)
  }
}
