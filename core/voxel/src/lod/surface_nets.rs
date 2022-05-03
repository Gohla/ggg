use crate::chunk::{ChunkMesh, ChunkSize, Vertex};
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::render::{copy_chunk_vertices, LodDraw};
use crate::surface_nets::SurfaceNets;
use crate::volume::Volume;

// Settings

#[derive(Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SurfaceNetsExtractorSettings {}

// Extractor

#[derive(Default, Copy, Clone)]
pub struct SurfaceNetsExtractor<C: ChunkSize> {
  surface_nets: SurfaceNets<C>,
  _settings: SurfaceNetsExtractorSettings,
}

impl<C: ChunkSize> LodExtractor<C> for SurfaceNetsExtractor<C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; SurfaceNets::<C>::SHARED_INDICES_SIZE]:,
{
  type Chunk = SurfaceNetsLodChunkMesh;

  #[inline]
  fn extract<V: Volume>(&self, _total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    let min = aabb.min();
    let step = aabb.step::<C>();
    let chunk_samples = volume.sample_chunk(min, step);
    self.surface_nets.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
  }
}

impl<C: ChunkSize> SurfaceNetsExtractor<C> {
  #[inline]
  pub fn new(surface_nets: SurfaceNets<C>, settings: SurfaceNetsExtractorSettings) -> Self {
    Self { surface_nets, _settings: settings }
  }
}

// Chunk vertices

#[repr(transparent)]
#[derive(Default, Clone, Debug)]
pub struct SurfaceNetsLodChunkMesh {
  pub regular: ChunkMesh,
}

impl SurfaceNetsLodChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(regular: ChunkMesh) -> Self {
    Self { regular }
  }
}

impl LodChunkMesh for SurfaceNetsLodChunkMesh {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
  }
}
