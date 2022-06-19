use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
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

impl<C: ChunkSize, V: Volume> LodExtractor<C, V> for MarchingCubesExtractor<C> {
  type Chunk = MarchingCubesLodChunkMesh;
  type JobInput = MarchingCubesJobInput;
  type DependencyKey = ();
  type DependenciesIntoIterator = [(Self::DependencyKey, LodJob<V, Self::JobInput, Self::DependenciesIntoIterator>); 1];

  #[inline]
  fn create_job(
    &self,
    _total_size: u32,
    aabb: AABB,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIntoIterator) {
    let input = MarchingCubesJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies = [((), LodJob::new_sample(aabb, volume))];
    (input, dependencies)
  }

  #[inline]
  fn run_job(
    &self,
    mut input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    if let (_, LodJobOutput::Sample(chunk_samples)) = &dependency_outputs[0] {
      self.marching_cubes.extract_chunk(input.aabb.min, input.aabb.step::<C>(), chunk_samples, &mut input.empty_lod_chunk_mesh.regular);
      input.empty_lod_chunk_mesh
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
  aabb: AABB,
  empty_lod_chunk_mesh: MarchingCubesLodChunkMesh,
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
