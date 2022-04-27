use crate::chunk::{ChunkSize, ChunkVertices, Vertex};
use crate::lod::aabb::AABB;
use crate::lod::chunk_vertices::LodChunkVertices;
use crate::lod::extract::LodExtractor;
use crate::lod::render::{copy_chunk_vertices, LodDraw, LodRenderDataUpdater};
use crate::surface_nets::SurfaceNets;
use crate::volume::Volume;

// Extractor

#[derive(Default, Copy, Clone)]
pub struct SurfaceNetsExtractor<C: ChunkSize> {
  surface_nets: SurfaceNets<C>,
}

impl<C: ChunkSize> LodExtractor<C> for SurfaceNetsExtractor<C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; SurfaceNets::<C>::SHARED_INDICES_SIZE]:,
{
  type Chunk = SurfaceNetsLodChunkVertices;

  #[inline]
  fn extract<V: Volume>(&self, _total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    let min = aabb.min();
    let step = aabb.step::<C>();
    let chunk_samples = volume.sample_chunk(min, step);
    self.surface_nets.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
  }
}

// Render updater

#[derive(Default, Copy, Clone, Debug)]
pub struct SurfaceNetsLodRendererSettings {}

#[derive(Default, Copy, Clone, Debug)]
pub struct SurfaceNetsLodRenderDataUpdater {
  pub settings: SurfaceNetsLodRendererSettings,
}

impl LodRenderDataUpdater for SurfaceNetsLodRenderDataUpdater {
  type Chunk = SurfaceNetsLodChunkVertices;

  fn update_chunk(&mut self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
  }
}

impl SurfaceNetsLodRenderDataUpdater {
  pub fn new(settings: SurfaceNetsLodRendererSettings) -> Self {
    Self { settings }
  }
}

// Chunk vertices

#[repr(transparent)]
#[derive(Default, Clone, Debug)]
pub struct SurfaceNetsLodChunkVertices {
  pub regular: ChunkVertices,
}

impl SurfaceNetsLodChunkVertices {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(regular: ChunkVertices) -> Self {
    Self { regular }
  }
}

impl LodChunkVertices for SurfaceNetsLodChunkVertices {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
  }
}
