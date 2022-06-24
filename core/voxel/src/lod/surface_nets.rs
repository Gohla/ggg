use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::{AABB, AABBSized};
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
    root_size: u32,
    aabb: AABBSized,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>) {
    let input = SurfaceNetsJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies_iterator = SurfaceNetsJobDependenciesIterator::new(root_size, aabb, volume, self.settings);
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
      let min = aabb.minimum_point();
      let step = aabb.step::<C>();
      let min_x = aabb.sibling_positive_x().map(|aabb| aabb.minimum_point());
      let min_y = aabb.sibling_positive_y().map(|aabb| aabb.minimum_point());
      let min_z = aabb.sibling_positive_z().map(|aabb| aabb.minimum_point());
      // Regular
      self.surface_nets.extract_chunk(min, step, &chunk_samples, &mut chunk.regular);
      // Positive X border
      if let Some(chunk_samples_x) = &chunk_samples_x {
        if let LodJobOutput::Sample(chunk_samples_x) = chunk_samples_x.borrow() {
          self.surface_nets_lod.extract_border_x(step, min, &chunk_samples, min_x.unwrap(), chunk_samples_x, &mut chunk.border_x_chunk);
        }
      }
      // Positive Y border
      if let Some(chunk_samples_y) = &chunk_samples_y {
        if let LodJobOutput::Sample(chunk_samples_y) = chunk_samples_y.borrow() {
          self.surface_nets_lod.extract_border_y(step, min, &chunk_samples, min_y.unwrap(), chunk_samples_y, &mut chunk.border_y_chunk);
        }
      }
      // Positive Z border
      if let Some(chunk_samples_z) = &chunk_samples_z {
        if let LodJobOutput::Sample(chunk_samples_z) = chunk_samples_z.borrow() {
          self.surface_nets_lod.extract_border_z(step, min, &chunk_samples, min_z.unwrap(), chunk_samples_z, &mut chunk.border_z_chunk);
        }
      }
      // Positive XY border
      if let (Some(chunk_samples_x), Some(chunk_samples_y), Some(chunk_samples_xy)) = (&chunk_samples_x, &chunk_samples_y, &chunk_samples_xy) {
        if let (LodJobOutput::Sample(chunk_samples_x), LodJobOutput::Sample(chunk_samples_y), LodJobOutput::Sample(chunk_samples_xy)) = (chunk_samples_x.borrow(), chunk_samples_y.borrow(), chunk_samples_xy.borrow()) {
          let min_xy = aabb.sibling_positive_xy().unwrap().minimum_point();
          self.surface_nets_lod.extract_border_xy(step, min, &chunk_samples, min_x.unwrap(), chunk_samples_x, min_y.unwrap(), chunk_samples_y, min_xy, chunk_samples_xy, &mut chunk.border_xy_chunk);
        }
      }
      // Positive YZ border
      if let (Some(chunk_samples_y), Some(chunk_samples_z), Some(chunk_samples_yz)) = (&chunk_samples_y, &chunk_samples_z, &chunk_samples_yz) {
        if let (LodJobOutput::Sample(chunk_samples_y), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_yz)) = (chunk_samples_y.borrow(), chunk_samples_z.borrow(), chunk_samples_yz.borrow()) {
          let min_yz = aabb.sibling_positive_yz().unwrap().minimum_point();
          self.surface_nets_lod.extract_border_yz(step, min, &chunk_samples, min_y.unwrap(), chunk_samples_y, min_z.unwrap(), chunk_samples_z, min_yz, chunk_samples_yz, &mut chunk.border_yz_chunk);
        }
      }
      // Positive XZ border
      if let (Some(chunk_samples_x), Some(chunk_samples_z), Some(chunk_samples_xz)) = (&chunk_samples_x, &chunk_samples_z, &chunk_samples_xz) {
        if let (LodJobOutput::Sample(chunk_samples_x), LodJobOutput::Sample(chunk_samples_z), LodJobOutput::Sample(chunk_samples_xz)) = (chunk_samples_x.borrow(), chunk_samples_z.borrow(), chunk_samples_xz.borrow()) {
          let min_xz = aabb.sibling_positive_xz().unwrap().minimum_point();
          self.surface_nets_lod.extract_border_xz(step, min, &chunk_samples, min_x.unwrap(), chunk_samples_x, min_z.unwrap(), chunk_samples_z, min_xz, chunk_samples_xz, &mut chunk.border_xz_chunk);
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
  aabb: AABBSized,
  empty_lod_chunk_mesh: SurfaceNetsLodChunkMesh,
}


// Job dependencies iterator

pub struct SurfaceNetsJobDependenciesIterator<C, V> {
  sample_regular: bool,
  sample_x: bool,
  sample_y: bool,
  sample_z: bool,
  sample_xy: bool,
  sample_yz: bool,
  sample_xz: bool,
  aabb: AABB,
  volume: V,
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize, V: Volume> SurfaceNetsJobDependenciesIterator<C, V> {
  #[inline]
  fn new(root_size: u32, aabb: AABBSized, volume: V, settings: SurfaceNetsExtractorSettings) -> Self {
    let maximum_point = aabb.maximum_point();
    let has_x_sibling = maximum_point.x < root_size;
    let has_y_sibling = maximum_point.y < root_size;
    let has_z_sibling = maximum_point.z < root_size;
    Self {
      sample_regular: settings.extract_regular_chunks,
      sample_x: has_x_sibling && (settings.extract_border_x_chunks || settings.extract_border_xy_chunks || settings.extract_border_xz_chunks),
      sample_y: has_y_sibling && (settings.extract_border_y_chunks || settings.extract_border_xy_chunks || settings.extract_border_yz_chunks),
      sample_z: has_z_sibling && (settings.extract_border_z_chunks || settings.extract_border_yz_chunks || settings.extract_border_xz_chunks),
      sample_xy: has_x_sibling && has_y_sibling && settings.extract_border_xy_chunks,
      sample_yz: has_y_sibling && has_z_sibling && settings.extract_border_yz_chunks,
      sample_xz: has_x_sibling && has_z_sibling && settings.extract_border_xz_chunks,
      aabb: aabb.inner,
      volume,
      _chunk_size_phantom: PhantomData::default(),
    }
  }

  #[inline]
  fn count(&self) -> usize {
    self.sample_regular as usize
      + self.sample_x as usize
      + self.sample_y as usize
      + self.sample_z as usize
      + self.sample_xy as usize
      + self.sample_yz as usize
      + self.sample_xz as usize
  }
}

impl<C: ChunkSize, V: Volume> Iterator for SurfaceNetsJobDependenciesIterator<C, V> {
  type Item = (SampleKind, LodJob<C, V, SurfaceNetsExtractor<C>>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    use SampleKind::*;
    // Regular
    if self.sample_regular {
      self.sample_regular = false;
      return Some((Regular, LodJob::new_sample(self.aabb, self.volume.clone())))
    }
    // Positive X
    if self.sample_x {
      self.sample_x = false;
      return Some((X, LodJob::new_sample(self.aabb.sibling_positive_x().unwrap(), self.volume.clone())));
    }
    // Positive Y
    if self.sample_y {
      self.sample_y = false;
      return Some((Y, LodJob::new_sample(self.aabb.sibling_positive_y().unwrap(), self.volume.clone())));
    }
    // Positive Z
    if self.sample_z {
      self.sample_z = false;
      return Some((Z, LodJob::new_sample(self.aabb.sibling_positive_z().unwrap(), self.volume.clone())));
    }
    // Positive XY
    if self.sample_xy {
      self.sample_xy = false;
      return Some((XY, LodJob::new_sample(self.aabb.sibling_positive_xy().unwrap(), self.volume.clone())));
    }
    // Positive YZ
    if self.sample_yz {
      self.sample_yz = false;
      return Some((YZ, LodJob::new_sample(self.aabb.sibling_positive_yz().unwrap(), self.volume.clone())));
    }
    // Positive XZ
    if self.sample_xz {
      self.sample_xz = false;
      return Some((XZ, LodJob::new_sample(self.aabb.sibling_positive_xz().unwrap(), self.volume.clone())));
    }
    None
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let count = self.count();
    (count, Some(count))
  }
}

impl<C: ChunkSize, V: Volume> ExactSizeIterator for SurfaceNetsJobDependenciesIterator<C, V> {
  #[inline]
  fn len(&self) -> usize { self.count() }
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

