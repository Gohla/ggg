use std::borrow::Borrow;

use job_queue::{Dependencies, DependencyOutputs, JobQueue, SendError};

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::{LodJobInput, LodJobKey, LodJobOutput};
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
  type JobDepKey = SampleKind;

  #[inline]
  fn create_jobs<V: Volume, const DS: usize>(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    lod_chunk_mesh: Self::Chunk,
    job_queue: &JobQueue<LodJobKey, Self::JobDepKey, LodJobInput<V, Self::Chunk>, LodJobOutput<ChunkSamples<C>, Self::Chunk>, DS>,
  ) -> Result<(), SendError<()>> {
    let min = aabb.min;
    let max = aabb.max_point();
    let size = aabb.size;

    // Gather dependencies to sample jobs.
    let mut dependencies = Dependencies::new();
    // Regular
    {
      let key = LodJobKey::Sample(aabb);
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::Regular, key));
    }
    // Positive X
    let make_x_border = max.x < total_size;
    if make_x_border && (self.settings.extract_border_x_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_xz_chunks) {
      let min_x = {
        let mut min = min;
        min.x = max.x;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_x, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::X, key));
    }
    // Positive Y
    let make_y_border = max.y < total_size;
    if make_y_border && (self.settings.extract_border_y_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_yz_chunks) {
      let min_y = {
        let mut min = min;
        min.y = max.y;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_y, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::Y, key));
    }
    // Positive Z
    let make_z_border = max.z < total_size;
    if make_z_border && (self.settings.extract_border_z_chunks || self.settings.extract_border_yz_chunks || self.settings.extract_border_xz_chunks) {
      let min_z = {
        let mut min = min;
        min.z = max.z;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_z, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::Z, key));
    }
    // Positive XY
    if make_x_border && make_y_border && self.settings.extract_border_xy_chunks {
      let min_xy = {
        let mut min = min;
        min.x = max.x;
        min.y = max.y;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_xy, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::XY, key));
    }
    // Positive YZ
    if make_y_border && make_z_border && self.settings.extract_border_yz_chunks {
      let min_yz = {
        let mut min = min;
        min.y = max.y;
        min.z = max.z;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_yz, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::YZ, key));
    }
    // Positive XZ
    if make_x_border && make_z_border && self.settings.extract_border_xz_chunks {
      let min_xz = {
        let mut min = min;
        min.x = max.x;
        min.z = max.z;
        min
      };
      let key = LodJobKey::Sample(AABB::new_unchecked(min_xz, size));
      job_queue.add_job(key, LodJobInput::Sample(volume.clone()))?;
      dependencies.push((SampleKind::XZ, key));
    }

    job_queue.add_job_with_dependencies(LodJobKey::Mesh(aabb), dependencies, LodJobInput::Mesh { total_size, lod_chunk_mesh })?;

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
    // Gather samples
    let mut chunk_samples = None;
    let mut chunk_samples_x = None;
    let mut chunk_samples_y = None;
    let mut chunk_samples_z = None;
    let mut chunk_samples_xy = None;
    let mut chunk_samples_yz = None;
    let mut chunk_samples_xz = None;
    for (neighbor, output) in dependency_outputs {
      match neighbor {
        SampleKind::Regular => chunk_samples = Some(output),
        SampleKind::X => chunk_samples_x = Some(output),
        SampleKind::Y => chunk_samples_y = Some(output),
        SampleKind::Z => chunk_samples_z = Some(output),
        SampleKind::XY => chunk_samples_xy = Some(output),
        SampleKind::YZ => chunk_samples_yz = Some(output),
        SampleKind::XZ => chunk_samples_xz = Some(output),
      }
    }
    if chunk_samples.is_none() {
      return;
    }
    let chunk_samples = chunk_samples.unwrap();
    if let LodJobOutput::Sample(chunk_samples) = chunk_samples.borrow() {
      // Extract
      let min = aabb.min;
      let max = aabb.max_point();
      let step = aabb.step::<C>();
      let min_x = { // TODO: reduce code duplication?
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
      // Regular
      self.surface_nets.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
      // Positive X border
      if let Some(chunk_samples_x) = &chunk_samples_x {
        if let LodJobOutput::Sample(chunk_samples_x) = chunk_samples_x.borrow() {
          self.surface_nets_lod.extract_border_x(step, min, &chunk_samples, min_x, chunk_samples_x, &mut chunk.border_x_chunk);
        }
      }
      // Positive Y border
      if let Some(chunk_samples_y) = &chunk_samples_y {
        if let LodJobOutput::Sample(chunk_samples_y) = chunk_samples_y.borrow() {
          self.surface_nets_lod.extract_border_y(step, min, &chunk_samples, min_y, chunk_samples_y, &mut chunk.border_y_chunk);
        }
      }
      // Positive Z border
      if let Some(chunk_samples_z) = &chunk_samples_z {
        if let LodJobOutput::Sample(chunk_samples_z) = chunk_samples_z.borrow() {
          self.surface_nets_lod.extract_border_z(step, min, &chunk_samples, min_z, chunk_samples_z, &mut chunk.border_z_chunk);
        }
      }
      // Positive XY border
      if let (Some(chunk_samples_x), Some(chunk_samples_y), Some(chunk_samples_xy)) = (&chunk_samples_x, &chunk_samples_y, &chunk_samples_xy) {
        if let (LodJobOutput::Sample(chunk_samples_x), LodJobOutput::Sample(chunk_samples_y), LodJobOutput::Sample(chunk_samples_xy)) = (chunk_samples_x.borrow(), chunk_samples_y.borrow(), chunk_samples_xy.borrow()) {
          let min_xy = {
            let mut min = min;
            min.x = max.x;
            min.y = max.y;
            min
          };
          self.surface_nets_lod.extract_border_xy(step, min, &chunk_samples, min_x, chunk_samples_x, min_y, chunk_samples_y, min_xy, chunk_samples_xy, &mut chunk.border_xy_chunk);
        }
      }
      // Positive YZ border
      if let (Some(chunk_samples_y), Some(chunk_samples_z), Some(chunk_samples_yz)) = (&chunk_samples_y, &chunk_samples_z, &chunk_samples_yz) {
        if let (LodJobOutput::Sample(chunk_samples_y), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_yz)) = (chunk_samples_y.borrow(), chunk_samples_z.borrow(), chunk_samples_yz.borrow()) {
          let min_yz = {
            let mut min = min;
            min.y = max.y;
            min.z = max.z;
            min
          };
          self.surface_nets_lod.extract_border_yz(step, min, &chunk_samples, min_y, chunk_samples_y, min_z, chunk_samples_z, min_yz, chunk_samples_yz, &mut chunk.border_yz_chunk);
        }
      }
      // Positive XZ border
      if let (Some(chunk_samples_x), Some(chunk_samples_z), Some(chunk_samples_xz)) = (&chunk_samples_x, &chunk_samples_z, &chunk_samples_xz) {
        if let (LodJobOutput::Sample(chunk_samples_x), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_xz)) = (chunk_samples_x.borrow(), chunk_samples_z.borrow(), chunk_samples_xz.borrow()) {
          let min_xz = {
            let mut min = min;
            min.x = max.x;
            min.z = max.z;
            min
          };
          self.surface_nets_lod.extract_border_xz(step, min, &chunk_samples, min_x, chunk_samples_x, min_z, chunk_samples_z, min_xz, chunk_samples_xz, &mut chunk.border_xz_chunk);
        }
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

// Sample kind

#[derive(Copy, Clone, Debug)]
pub enum SampleKind {
  Regular,
  X,
  Y,
  Z,
  XY,
  YZ,
  XZ,
}
