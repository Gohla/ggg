use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::MaybeCompressedChunkSampleArray;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::{Aabb, AabbWithSize};
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::{LodExtractor, NeighborDepths};
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
    aabb: AabbWithSize,
    neighbor_depths: NeighborDepths,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>) {
    let input = SurfaceNetsJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies_iterator = SurfaceNetsJobDependenciesIterator::new(aabb.inner, neighbor_depths, volume, self.settings);
    (input, dependencies_iterator)
  }

  #[inline]
  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<MaybeCompressedChunkSampleArray<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    let mut chunk = input.empty_lod_chunk_mesh;
    // Gather samples
    let mut chunk_samples = None;
    let mut chunk_samples_x = None;
    let mut chunk_samples_x_front = None;
    let mut chunk_samples_x_front_y = None;
    let mut chunk_samples_x_back = None;
    let mut chunk_samples_x_back_y = None;
    let mut chunk_samples_y = None;
    let mut chunk_samples_z = None;
    let mut chunk_samples_xy = None;
    let mut chunk_samples_yz = None;
    let mut chunk_samples_xz = None;
    for (neighbor, output) in dependency_outputs {
      match neighbor {
        SampleKind::Regular => chunk_samples = Some(output),
        SampleKind::X => chunk_samples_x = Some(output),
        SampleKind::XFront => chunk_samples_x_front = Some(output),
        SampleKind::XFrontY => chunk_samples_x_front_y = Some(output),
        SampleKind::XBack => chunk_samples_x_back = Some(output),
        SampleKind::XBackY => chunk_samples_x_back_y = Some(output),
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
      self.surface_nets.extract_chunk_from_maybe_compressed_samples(min, step, &chunk_samples, &mut chunk.regular);
      // Positive X border
      if let (
        Some(LodJobOutput::Sample(chunk_samples_x_front))
        , Some(LodJobOutput::Sample(chunk_samples_x_front_y))
        , Some(LodJobOutput::Sample(chunk_samples_x_back))
        , Some(LodJobOutput::Sample(chunk_samples_x_back_y))
      ) = (
        chunk_samples_x_front.map(|s| s.borrow())
        , chunk_samples_x_front_y.map(|s| s.borrow())
        , chunk_samples_x_back.map(|s| s.borrow())
        , chunk_samples_x_back_y.map(|s| s.borrow())
      ) {
        let x_subdivided = aabb.sibling_positive_x().map(|aabb| aabb.subdivide_array());
        let min_x_front = x_subdivided.map(|subdivided| subdivided[1].minimum_point(aabb.root_size)).unwrap();
        let min_x_front_y = x_subdivided.map(|subdivided| subdivided[3].minimum_point(aabb.root_size)).unwrap();
        let min_x_back = x_subdivided.map(|subdivided| subdivided[5].minimum_point(aabb.root_size)).unwrap();
        let min_x_back_y = x_subdivided.map(|subdivided| subdivided[7].minimum_point(aabb.root_size)).unwrap();
        self.surface_nets_lod.extract_border_x_hires(step, min, &chunk_samples, step * 2, min_x_front, chunk_samples_x_front, min_x_front_y, chunk_samples_x_front_y, min_x_back, chunk_samples_x_back, min_x_back_y, chunk_samples_x_back_y, &mut chunk.border_x_chunk);
      } else if let Some(chunk_samples_x) = &chunk_samples_x {
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
  aabb: AabbWithSize,
  empty_lod_chunk_mesh: SurfaceNetsLodChunkMesh,
}


// Job dependencies iterator

pub struct SurfaceNetsJobDependenciesIterator<C, V> {
  regular_aabb: Option<Aabb>,
  x_aabb: Option<Aabb>,
  x_front_aabb: Option<Aabb>,
  x_front_y_aabb: Option<Aabb>,
  x_back_aabb: Option<Aabb>,
  x_back_y_aabb: Option<Aabb>,
  y_aabb: Option<Aabb>,
  z_aabb: Option<Aabb>,
  xy_aabb: Option<Aabb>,
  yz_aabb: Option<Aabb>,
  xz_aabb: Option<Aabb>,
  volume: V,
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize, V: Volume> SurfaceNetsJobDependenciesIterator<C, V> {
  #[inline]
  fn new(aabb: Aabb, neighbor_depths: NeighborDepths, volume: V, settings: SurfaceNetsExtractorSettings) -> Self {
    let depth = aabb.depth();
    let x_aabb = aabb.sibling_positive_x();
    let has_x_sibling = neighbor_depths.x != 0;
    let x_sibling_hires = neighbor_depths.x > depth;
    let x_subdivided = x_aabb.map(|aabb| aabb.subdivide());
    let x_front_aabb = x_subdivided.as_ref().map(|s| s.x);
    let x_front_y_aabb = x_subdivided.as_ref().map(|s| s.xy);
    let x_back_aabb = x_subdivided.as_ref().map(|s| s.xz);
    let x_back_y_aabb = x_subdivided.as_ref().map(|s| s.xyz);
    let y_aabb = aabb.sibling_positive_y();
    let has_y_sibling = neighbor_depths.y != 0;
    let z_aabb = aabb.sibling_positive_z();
    let has_z_sibling = neighbor_depths.z != 0;
    Self {
      regular_aabb: settings.extract_regular_chunks.then_some(aabb),
      x_aabb: ((!x_sibling_hires && settings.extract_border_x_chunks) || settings.extract_border_xy_chunks || settings.extract_border_xz_chunks).then_some(x_aabb).flatten(),
      x_front_aabb: (x_sibling_hires && settings.extract_border_x_chunks).then_some(x_front_aabb).flatten(),
      x_front_y_aabb: (x_sibling_hires && settings.extract_border_x_chunks).then_some(x_front_y_aabb).flatten(),
      x_back_aabb: (x_sibling_hires && settings.extract_border_x_chunks).then_some(x_back_aabb).flatten(),
      x_back_y_aabb: (x_sibling_hires && settings.extract_border_x_chunks).then_some(x_back_y_aabb).flatten(),
      y_aabb: (settings.extract_border_y_chunks || settings.extract_border_xy_chunks || settings.extract_border_yz_chunks).then_some(y_aabb).flatten(),
      z_aabb: (settings.extract_border_z_chunks || settings.extract_border_yz_chunks || settings.extract_border_xz_chunks).then_some(z_aabb).flatten(),
      xy_aabb: (has_x_sibling && has_y_sibling && settings.extract_border_xy_chunks).then(|| aabb.sibling_positive_xy()).flatten(),
      yz_aabb: (has_y_sibling && has_z_sibling && settings.extract_border_yz_chunks).then(|| aabb.sibling_positive_yz()).flatten(),
      xz_aabb: (has_x_sibling && has_z_sibling && settings.extract_border_xz_chunks).then(|| aabb.sibling_positive_xz()).flatten(),
      volume,
      _chunk_size_phantom: PhantomData::default(),
    }
  }

  #[inline]
  fn count(&self) -> usize {
    self.regular_aabb.is_some() as usize
      + self.x_aabb.is_some() as usize
      + self.y_aabb.is_some() as usize
      + self.z_aabb.is_some() as usize
      + self.xy_aabb.is_some() as usize
      + self.yz_aabb.is_some() as usize
      + self.xz_aabb.is_some() as usize
  }
}

impl<C: ChunkSize, V: Volume> Iterator for SurfaceNetsJobDependenciesIterator<C, V> {
  type Item = (SampleKind, LodJob<C, V, SurfaceNetsExtractor<C>>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    use SampleKind::*;
    if let Some(aabb) = self.regular_aabb.take() {
      return Some((Regular, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.x_aabb.take() {
      return Some((X, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.x_front_aabb.take() {
      return Some((XFront, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.x_front_y_aabb.take() {
      return Some((XFrontY, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.x_back_aabb.take() {
      return Some((XBack, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.x_back_y_aabb.take() {
      return Some((XBackY, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.y_aabb.take() {
      return Some((Y, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.z_aabb.take() {
      return Some((Z, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.xy_aabb.take() {
      return Some((XY, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.yz_aabb.take() {
      return Some((YZ, LodJob::new_sample(aabb, self.volume.clone())))
    }
    if let Some(aabb) = self.xz_aabb.take() {
      return Some((XZ, LodJob::new_sample(aabb, self.volume.clone())))
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
  XFront,
  XFrontY,
  XBack,
  XBackY,
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

