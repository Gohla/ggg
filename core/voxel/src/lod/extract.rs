use job_queue::{DependencyOutputs, DepKey, JobQueue, SendError};

use crate::chunk::mesh::Vertex;
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::octmap::{LodJobInput, LodJobKey, LodJobOutput};
use crate::lod::render::LodDraw;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize>: Clone + Send + Sync + 'static {
  type Chunk: LodChunkMesh + Send + Sync + 'static;
  type JobDepKey: DepKey;

  fn create_jobs<V: Volume, const DS: usize>(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    lod_chunk_mesh: Self::Chunk,
    job_queue: &JobQueue<LodJobKey, Self::JobDepKey, LodJobInput<V, Self::Chunk>, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>,
  ) -> Result<(), SendError<()>>;

  fn run_job<const DS: usize>(
    &self,
    total_size: u32,
    aabb: AABB,
    dependency_outputs: DependencyOutputs<Self::JobDepKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>,
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

impl<C: ChunkSize, CM: LodChunkMesh, JD: DepKey, T: LodExtractor<C, Chunk=CM, JobDepKey=JD> + ?Sized> LodExtractor<C> for Box<T> {
  type Chunk = CM;
  type JobDepKey = JD;


  #[inline]
  fn create_jobs<V: Volume, const DS: usize>(&self, total_size: u32, aabb: AABB, volume: V, lod_chunk_mesh: Self::Chunk, job_queue: &JobQueue<LodJobKey, Self::JobDepKey, LodJobInput<V, Self::Chunk>, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>) -> Result<(), SendError<()>> {
    (**self).create_jobs::<V, DS>(total_size, aabb, volume, lod_chunk_mesh, job_queue)
  }

  #[inline]
  fn run_job<const DS: usize>(&self, total_size: u32, aabb: AABB, dependency_outputs: DependencyOutputs<Self::JobDepKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>, chunk: &mut Self::Chunk) {
    (**self).run_job(total_size, aabb, dependency_outputs, chunk);
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    (**self).update_render_data(chunk, vertices, indices, draws);
  }
}
