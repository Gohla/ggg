use job_queue::{DepKey, In};

use crate::chunk::mesh::Vertex;
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AabbWithSize;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::octmap::{LodJob, LodJobOutput};
use crate::lod::render::LodDraw;
use crate::volume::Volume;

/// Extracts chunks of vertices with LOD from a volume.
pub trait LodExtractor<C: ChunkSize>: Clone + Send + Sync + 'static {
  type Chunk: LodChunkMesh + Send + Sync + 'static;
  type JobInput: In;
  type DependencyKey: DepKey;
  type DependenciesIterator<V: Volume>: Iterator<Item=(Self::DependencyKey, LodJob<C, V, Self>)> + Send + 'static;

  fn create_job<V: Volume>(
    &self,
    aabb: AabbWithSize,
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

impl<C: ChunkSize> LodExtractor<C> for () {
  type Chunk = ();
  type JobInput = ();
  type DependencyKey = ();
  type DependenciesIterator<V: Volume> = std::iter::Empty<(Self::DependencyKey, LodJob<C, V, Self>)>;
  #[inline]
  fn create_job<V: Volume>(&self, _aabb: AabbWithSize, _volume: V, _empty_lod_chunk_mesh: Self::Chunk) -> (Self::JobInput, Self::DependenciesIterator<V>) { ((), std::iter::empty()) }
  #[inline]
  fn run_job(&self, _input: Self::JobInput, _dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)]) -> Self::Chunk { () }
  #[inline]
  fn update_render_data(&self, _chunk: &Self::Chunk, _vertices: &mut Vec<Vertex>, _indices: &mut Vec<u16>, _draws: &mut Vec<LodDraw>) {}
}

impl LodChunkMesh for () {
  #[inline]
  fn is_empty(&self) -> bool { false }
  #[inline]
  fn clear(&mut self) {}
}
