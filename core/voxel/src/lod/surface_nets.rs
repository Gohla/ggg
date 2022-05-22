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
  pub extract_border_y_chunks: bool,
  pub extract_border_z_chunks: bool,
  pub extract_border_xy_chunks: bool,
  pub extract_border_yz_chunks: bool,
  pub extract_border_xz_chunks: bool,
}

impl Default for SurfaceNetsExtractorSettings {
  fn default() -> Self {
    Self {
      extract_regular_chunks: true,
      extract_border_x_chunks: true,
      extract_border_y_chunks: true,
      extract_border_z_chunks: true,
      extract_border_xy_chunks: false,
      extract_border_yz_chunks: false,
      extract_border_xz_chunks: false,
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

    let min_x = {
      let mut min = min;
      min.x = max.x;
      min
    };
    let min_y = {
      let mut min = min;
      min.y = max.y;
      min
    };
    let min_z = {
      let mut min = min;
      min.z = max.z;
      min
    };

    let sample_x = self.settings.extract_border_x_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_xz_chunks;
    let chunk_samples_x = (sample_x && max.x < total_size).then(|| {
      volume.sample_chunk(min_x, step)
    });
    let sample_y = self.settings.extract_border_y_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_yz_chunks;
    let chunk_samples_y = (sample_y && max.y < total_size).then(|| {
      volume.sample_chunk(min_y, step)
    });
    let sample_z = self.settings.extract_border_z_chunks || self.settings.extract_border_yz_chunks || self.settings.extract_border_xz_chunks;
    let chunk_samples_z = (sample_z && max.z < total_size).then(|| {
      volume.sample_chunk(min_z, step)
    });

    if self.settings.extract_border_x_chunks {
      if let Some(chunk_samples_x) = &chunk_samples_x {
        self.surface_nets_lod.extract_border_x(step, min, &chunk_samples, min_x, chunk_samples_x, &mut chunk.border_x_chunk);
      }
    }
    if self.settings.extract_border_y_chunks {
      if let Some(chunk_samples_y) = &chunk_samples_y {
        self.surface_nets_lod.extract_border_y(step, min, &chunk_samples, min_y, chunk_samples_y, &mut chunk.border_y_chunk);
      }
    }
    if self.settings.extract_border_z_chunks {
      if let Some(chunk_samples_z) = &chunk_samples_z {
        self.surface_nets_lod.extract_border_z(step, min, &chunk_samples, min_z, chunk_samples_z, &mut chunk.border_z_chunk);
      }
    }

    if self.settings.extract_border_xy_chunks {
      if let (Some(chunk_samples_x), Some(chunk_samples_y)) = (&chunk_samples_x, &chunk_samples_y) {
        let min_xy = {
          let mut min = min;
          min.x = max.x;
          min.y = max.y;
          min
        };
        let chunk_samples_xy = volume.sample_chunk(min_xy, step);
        self.surface_nets_lod.extract_border_xy(step, min, &chunk_samples, min_x, chunk_samples_x, min_y, chunk_samples_y, min_xy, &chunk_samples_xy, &mut chunk.border_xy_chunk);
      }
    }
    if self.settings.extract_border_yz_chunks {
      if let (Some(chunk_samples_y), Some(chunk_samples_z)) = &(&chunk_samples_y, &chunk_samples_z) {
        let min_yz = {
          let mut min = min;
          min.y = max.y;
          min.z = max.z;
          min
        };
        let chunk_samples_yz = volume.sample_chunk(min_yz, step);
        self.surface_nets_lod.extract_border_yz(step, min, &chunk_samples, min_y, chunk_samples_y, min_z, chunk_samples_z, min_yz, &chunk_samples_yz, &mut chunk.border_yz_chunk);
      }
    }
    if self.settings.extract_border_xz_chunks {
      if let (Some(chunk_samples_x), Some(chunk_samples_z)) = &(&chunk_samples_x, &chunk_samples_z) {
        let min_xz = {
          let mut min = min;
          min.x = max.x;
          min.z = max.z;
          min
        };
        let chunk_samples_xz = volume.sample_chunk(min_xz, step);
        self.surface_nets_lod.extract_border_xz(step, min, &chunk_samples, min_x, chunk_samples_x, min_z, chunk_samples_z, min_xz, &chunk_samples_xz, &mut chunk.border_xz_chunk);
      }
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
    if self.settings.extract_border_y_chunks {
      copy_chunk_vertices(&chunk.border_y_chunk, vertices, indices, draws);
    }
    if self.settings.extract_border_z_chunks {
      copy_chunk_vertices(&chunk.border_z_chunk, vertices, indices, draws);
    }
    if self.settings.extract_border_xy_chunks {
      copy_chunk_vertices(&chunk.border_xy_chunk, vertices, indices, draws);
    }
    if self.settings.extract_border_yz_chunks {
      copy_chunk_vertices(&chunk.border_yz_chunk, vertices, indices, draws);
    }
    if self.settings.extract_border_xz_chunks {
      copy_chunk_vertices(&chunk.border_xz_chunk, vertices, indices, draws);
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
  pub border_y_chunk: ChunkMesh,
  pub border_z_chunk: ChunkMesh,
  pub border_xy_chunk: ChunkMesh,
  pub border_yz_chunk: ChunkMesh,
  pub border_xz_chunk: ChunkMesh,
}

impl SurfaceNetsLodChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(
    regular: ChunkMesh,
    border_x_chunk: ChunkMesh,
    border_y_chunk: ChunkMesh,
    border_z_chunk: ChunkMesh,
    border_xy_chunk: ChunkMesh,
    border_yz_chunk: ChunkMesh,
    border_xz_chunk: ChunkMesh,
  ) -> Self {
    Self {
      regular,
      border_x_chunk,
      border_y_chunk,
      border_z_chunk,
      border_xy_chunk,
      border_yz_chunk,
      border_xz_chunk,
    }
  }
}

impl LodChunkMesh for SurfaceNetsLodChunkMesh {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty()
      && self.border_x_chunk.is_empty()
      && self.border_y_chunk.is_empty()
      && self.border_z_chunk.is_empty()
      && self.border_xy_chunk.is_empty()
      && self.border_yz_chunk.is_empty()
      && self.border_xz_chunk.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
    self.border_x_chunk.clear();
    self.border_y_chunk.clear();
    self.border_z_chunk.clear();
    self.border_xy_chunk.clear();
    self.border_yz_chunk.clear();
    self.border_xz_chunk.clear();
  }
}
