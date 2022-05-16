use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
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

  #[inline]
  fn extract<V: Volume>(&self, _total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    let min = aabb.min();
    let step = aabb.step::<C>();
    let chunk_samples = volume.sample_chunk(min, step);
    self.marching_cubes.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
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
