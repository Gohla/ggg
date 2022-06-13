use std::borrow::Borrow;
use std::sync::Arc;

use job_queue::{Dependencies, DependencyOutputs, JobQueue, SendError};

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::{LodJobInput, LodJobKey, LodJobOutput};
use crate::lod::render::{copy_chunk_vertices, LodDraw};
use crate::marching_cubes::MarchingCubes;
use crate::volume::Volume;

// Settings

#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MarchingCubesExtractorSettings {}

// Extractor

#[derive(Default, Copy, Clone)]
pub struct MarchingCubesExtractor<C: ChunkSize> {
  marching_cubes: MarchingCubes<C>,
  _settings: MarchingCubesExtractorSettings,
}

impl<C: ChunkSize> LodExtractor<C> for MarchingCubesExtractor<C> {
  type Chunk = MarchingCubesLodChunkVertices;
  type JobDepKey = ();

  #[inline]
  fn create_jobs<V: Volume, const DS: usize>(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    lod_chunk_mesh: Self::Chunk,
    job_queue: &JobQueue<LodJobKey, Self::JobDepKey, LodJobInput<V, Self::Chunk>, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>,
  ) -> Result<(), SendError<()>> {
    let sample_key = LodJobKey::Sample(aabb);
    job_queue.add_job(sample_key, LodJobInput::Sample(volume))?;
    job_queue.add_job_with_dependencies(LodJobKey::Mesh(aabb), Dependencies::from_elem(((), sample_key), 1), LodJobInput::Mesh { total_size, lod_chunk_mesh })?;
    Ok(())
  }

  #[inline]
  fn run_job<const DS: usize>(
    &self,
    _total_size: u32,
    aabb: AABB,
    dependency_outputs: DependencyOutputs<Self::JobDepKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>,
    chunk: &mut Self::Chunk,
  ) {
    let (_, output): &((), Arc<LodJobOutput<ChunkSamples<C>, Self::Chunk>>) = &dependency_outputs[0];
    if let LodJobOutput::Sample(chunk_samples) = output.borrow() {
      self.marching_cubes.extract_chunk(aabb.min, aabb.step::<C>(), chunk_samples, &mut chunk.regular);
    }
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
  }
}

impl<C: ChunkSize> MarchingCubesExtractor<C> {
  #[inline]
  pub fn new(marching_cubes: MarchingCubes<C>, settings: MarchingCubesExtractorSettings) -> Self {
    Self { marching_cubes, _settings: settings }
  }
}

// Chunk vertices

#[repr(transparent)]
#[derive(Default, Clone, Debug)]
pub struct MarchingCubesLodChunkVertices {
  pub regular: ChunkMesh,
}

impl MarchingCubesLodChunkVertices {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(regular: ChunkMesh) -> Self {
    Self { regular }
  }
}

impl LodChunkMesh for MarchingCubesLodChunkVertices {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
  }
}
