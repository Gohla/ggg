use std::borrow::Borrow;
use std::marker::PhantomData;

use ultraviolet::UVec3;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::{LodJob, LodJobOutput};
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
  #[inline]
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
  type JobInput = SurfaceNetsJobInput;
  type DependencyKey = SampleKind;
  type DependenciesIterator<V: Volume> = SurfaceNetsJobDependenciesIterator<C, V>;

  #[inline]
  fn create_job<V: Volume>(
    &self,
    total_size: u32,
    aabb: AABB,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>) {
    let aabb = AABBWithMax::new(aabb);
    let input = SurfaceNetsJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies_iterator = SurfaceNetsJobDependenciesIterator::new(total_size, aabb, volume, self.settings);
    (input, dependencies_iterator)
  }

  #[inline]
  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<ChunkSamples<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    let mut chunk = input.empty_lod_chunk_mesh;
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
      panic!("Missing regular sample dependency output");
    }
    let chunk_samples = chunk_samples.unwrap();
    if let LodJobOutput::Sample(chunk_samples) = chunk_samples.borrow() {
      // Extract
      let aabb = input.aabb;
      let min = aabb.inner.min;
      let step = aabb.inner.step::<C>();
      let min_x = aabb.min_x();
      let min_y = aabb.min_y();
      let min_z = aabb.min_z();
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
          self.surface_nets_lod.extract_border_xy(step, min, &chunk_samples, min_x, chunk_samples_x, min_y, chunk_samples_y, aabb.min_xy(), chunk_samples_xy, &mut chunk.border_xy_chunk);
        }
      }
      // Positive YZ border
      if let (Some(chunk_samples_y), Some(chunk_samples_z), Some(chunk_samples_yz)) = (&chunk_samples_y, &chunk_samples_z, &chunk_samples_yz) {
        if let (LodJobOutput::Sample(chunk_samples_y), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_yz)) = (chunk_samples_y.borrow(), chunk_samples_z.borrow(), chunk_samples_yz.borrow()) {
          self.surface_nets_lod.extract_border_yz(step, min, &chunk_samples, min_y, chunk_samples_y, min_z, chunk_samples_z, aabb.min_yz(), chunk_samples_yz, &mut chunk.border_yz_chunk);
        }
      }
      // Positive XZ border
      if let (Some(chunk_samples_x), Some(chunk_samples_z), Some(chunk_samples_xz)) = (&chunk_samples_x, &chunk_samples_z, &chunk_samples_xz) {
        if let (LodJobOutput::Sample(chunk_samples_x), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_xz)) = (chunk_samples_x.borrow(), chunk_samples_z.borrow(), chunk_samples_xz.borrow()) {
          self.surface_nets_lod.extract_border_xz(step, min, &chunk_samples, min_x, chunk_samples_x, min_z, chunk_samples_z, aabb.min_xz(), chunk_samples_xz, &mut chunk.border_xz_chunk);
        }
      }
    }
    chunk
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

// Job input

pub struct SurfaceNetsJobInput {
  aabb: AABBWithMax,
  empty_lod_chunk_mesh: SurfaceNetsLodChunkMesh,
}


// Job dependencies iterator

pub struct SurfaceNetsJobDependenciesIterator<C, V> {
  aabb: AABBWithMax,
  make_x_border: bool,
  make_y_border: bool,
  make_z_border: bool,
  volume: V,
  settings: SurfaceNetsExtractorSettings,
  stage: Option<SampleKind>,
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize, V: Volume> SurfaceNetsJobDependenciesIterator<C, V> {
  #[inline]
  fn new(total_size: u32, aabb: AABBWithMax, volume: V, settings: SurfaceNetsExtractorSettings) -> Self {
    Self {
      aabb,
      make_x_border: aabb.max.x < total_size,
      make_y_border: aabb.max.y < total_size,
      make_z_border: aabb.max.z < total_size,
      volume,
      settings,
      stage: Some(SampleKind::Regular),
      _chunk_size_phantom: PhantomData::default(),
    }
  }
}

impl<C: ChunkSize, V: Volume> Iterator for SurfaceNetsJobDependenciesIterator<C, V> {
  type Item = (SampleKind, LodJob<C, V, SurfaceNetsExtractor<C>>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    use SampleKind::*;
    if self.stage.is_none() { return None; }
    let size = self.aabb.inner.size;
    // Regular
    if self.stage == Some(Regular) {
      self.stage = Some(X);
      if self.settings.extract_regular_chunks {
        return Some((Regular, LodJob::new_sample(self.aabb.inner, self.volume.clone())))
      }
    }
    // Positive X
    if self.stage == Some(X) {
      self.stage = Some(Y);
      if self.make_x_border && (self.settings.extract_border_x_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_xz_chunks) {
        return Some((X, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_x(), size), self.volume.clone())));
      }
    }
    // Positive Y
    if self.stage == Some(Y) {
      self.stage = Some(Z);
      if self.make_y_border && (self.settings.extract_border_y_chunks || self.settings.extract_border_xy_chunks || self.settings.extract_border_yz_chunks) {
        return Some((Y, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_y(), size), self.volume.clone())));
      }
    }
    // Positive Z
    if self.stage == Some(Z) {
      self.stage = Some(XY);
      if self.make_z_border && (self.settings.extract_border_z_chunks || self.settings.extract_border_yz_chunks || self.settings.extract_border_xz_chunks) {
        return Some((Z, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_z(), size), self.volume.clone())));
      }
    }
    // Positive XY
    if self.stage == Some(XY) {
      self.stage = Some(YZ);
      if self.make_x_border && self.make_y_border && self.settings.extract_border_xy_chunks {
        return Some((XY, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_xy(), size), self.volume.clone())));
      }
    }
    // Positive YZ
    if self.stage == Some(YZ) {
      self.stage = Some(XZ);
      if self.make_y_border && self.make_z_border && self.settings.extract_border_yz_chunks {
        return Some((YZ, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_yz(), size), self.volume.clone())));
      }
    }
    // Positive XZ
    if self.stage == Some(XZ) {
      self.stage = None;
      if self.make_x_border && self.make_z_border && self.settings.extract_border_xz_chunks {
        return Some((XZ, LodJob::new_sample(AABB::new_unchecked(self.aabb.min_xz(), size), self.volume.clone())));
      }
    }
    None
  }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum SampleKind {
  Regular,
  X,
  Y,
  Z,
  XY,
  YZ,
  XZ,
}


// Chunk mesh

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


// AABB extensions

#[derive(Copy, Clone)]
struct AABBWithMax {
  inner: AABB,
  max: UVec3,
}

impl AABBWithMax {
  #[inline]
  fn new(aabb: AABB) -> Self {
    Self { inner: aabb, max: aabb.max_point() }
  }

  #[inline]
  fn min_x(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.x = self.max.x;
    min
  }

  #[inline]
  fn min_y(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.y = self.max.y;
    min
  }

  #[inline]
  fn min_z(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.z = self.max.z;
    min
  }

  #[inline]
  fn min_xy(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.x = self.max.x;
    min.y = self.max.y;
    min
  }

  #[inline]
  fn min_yz(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.y = self.max.y;
    min.z = self.max.z;
    min
  }

  #[inline]
  fn min_xz(&self) -> UVec3 {
    let mut min = self.inner.min;
    min.x = self.max.x;
    min.z = self.max.z;
    min
  }
}

