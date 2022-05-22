use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::render::{copy_chunk_vertices, LodDraw};
use crate::surface_nets::lod::SurfaceNetsLod;
use crate::surface_nets::SurfaceNets;
use crate::volume::Volume;

// Settings

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SurfaceNetsExtractorSettings {
  pub extract_regular_chunks: bool,
  pub extract_border_x_chunks: bool,
}

impl Default for SurfaceNetsExtractorSettings {
  fn default() -> Self {
    Self {
      extract_regular_chunks: true,
      extract_border_x_chunks: false,
    }
  }
}

// Extractor

#[derive(Default, Copy, Clone)]
pub struct SurfaceNetsExtractor<C: ChunkSize> {
  surface_nets: SurfaceNets<C>,
  surface_nets_lod: SurfaceNetsLod<C>,
  settings: SurfaceNetsExtractorSettings,
}

impl<C: ChunkSize> LodExtractor<C> for SurfaceNetsExtractor<C> {
  type Chunk = SurfaceNetsLodChunkMesh;

  #[inline]
  fn extract<V: Volume>(&self, total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    let min = aabb.min();
    let max = aabb.max();
    let step = aabb.step::<C>();
    let chunk_samples = volume.sample_chunk(min, step);
    if self.settings.extract_regular_chunks {
      self.surface_nets.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
    }
    if self.settings.extract_border_x_chunks && max.x < total_size {
      let mut min_b = min;
      min_b.x = max.x;
      let chunk_samples_b = volume.sample_chunk(min_b, step);
      self.surface_nets_lod.extract_border_x(step, min, &chunk_samples, min_b, &chunk_samples_b, &mut chunk.border_x_chunk);
    }
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    if self.settings.extract_regular_chunks {
      copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
    }
    if self.settings.extract_border_x_chunks {
      copy_chunk_vertices(&chunk.border_x_chunk, vertices, indices, draws);
    }
  }
}

impl<C: ChunkSize> SurfaceNetsExtractor<C> {
  #[inline]
  pub fn new(surface_nets: SurfaceNets<C>, surface_nets_lod: SurfaceNetsLod<C>, settings: SurfaceNetsExtractorSettings) -> Self {
    Self { surface_nets, surface_nets_lod, settings }
  }
}

// Chunk vertices

#[derive(Default, Clone, Debug)]
pub struct SurfaceNetsLodChunkMesh {
  pub regular: ChunkMesh,
  pub border_x_chunk: ChunkMesh,
}

impl SurfaceNetsLodChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(regular: ChunkMesh, border_x_chunk: ChunkMesh) -> Self {
    Self { regular, border_x_chunk }
  }
}

impl LodChunkMesh for SurfaceNetsLodChunkMesh {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty() && self.border_x_chunk.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
    self.border_x_chunk.clear();
  }
}
