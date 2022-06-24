use std::marker::PhantomData;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::{AABB, AABBSized};
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::{LodJob, LodJobOutput};
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
  type Chunk = MarchingCubesLodChunkMesh;
  type JobInput = MarchingCubesJobInput;
  type DependencyKey = ();
  type DependenciesIterator<V: Volume> = MarchingCubesJobDependenciesIterator<C, V>;

  #[inline]
  fn create_job<V: Volume>(
    &self,
    _root_size: u32,
    aabb: AABBSized,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>) {
    let input = MarchingCubesJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies = MarchingCubesJobDependenciesIterator::new(aabb.inner, volume);
    (input, dependencies)
  }

  #[inline]
  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    if let (_, LodJobOutput::Sample(chunk_samples)) = &dependency_outputs[0] {
      let MarchingCubesJobInput { aabb, empty_lod_chunk_mesh: mut chunk } = input;
      self.marching_cubes.extract_chunk(aabb.minimum_point(), aabb.step::<C>(), chunk_samples, &mut chunk.regular);
      chunk
    } else {
      panic!("Missing sample dependency output");
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


// Job input

pub struct MarchingCubesJobInput {
  aabb: AABBSized,
  empty_lod_chunk_mesh: MarchingCubesLodChunkMesh,
}


// Job dependencies iterator

pub struct MarchingCubesJobDependenciesIterator<C, V> {
  aabb: AABB,
  volume: Option<V>,
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize, V: Volume> MarchingCubesJobDependenciesIterator<C, V> {
  #[inline]
  fn new(aabb: AABB, volume: V) -> Self { Self { aabb, volume: Some(volume), _chunk_size_phantom: PhantomData::default() } }
}

impl<C: ChunkSize, V: Volume> Iterator for MarchingCubesJobDependenciesIterator<C, V> {
  type Item = ((), LodJob<C, V, MarchingCubesExtractor<C>>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let volume = self.volume.take();
    if let Some(volume) = volume {
      Some(((), LodJob::new_sample(self.aabb, volume)))
    } else {
      None
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) { match &self.volume { Some(_) => (1, Some(1)), None => (0, Some(0)) } }
}

impl<C: ChunkSize, V: Volume> ExactSizeIterator for MarchingCubesJobDependenciesIterator<C, V> {
  #[inline]
  fn len(&self) -> usize { match &self.volume { Some(_) => 1, None => 0 } }
}

// Chunk vertices

#[repr(transparent)]
#[derive(Default, Clone, Debug)]
pub struct MarchingCubesLodChunkMesh {
  pub regular: ChunkMesh,
}

impl MarchingCubesLodChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(regular: ChunkMesh) -> Self {
    Self { regular }
  }
}

impl LodChunkMesh for MarchingCubesLodChunkMesh {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
  }
}
