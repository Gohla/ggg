use std::marker::PhantomData;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::MaybeCompressedChunkSampleArray;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::{Aabb, AabbWithSize};
use crate::lod::chunk_mesh::LodChunkMesh;
use crate::lod::extract::{LodExtractor, NeighborDepths};
use crate::lod::octmap::{LodJob, LodJobOutput};
use crate::lod::render::{copy_chunk_vertices, LodDraw};
use crate::marching_cubes::MarchingCubes;
use crate::transvoxel::side::TransitionSide;
use crate::transvoxel::Transvoxel;
use crate::volume::{Sphere, SphereSettings, Volume};

// Settings

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TransvoxelExtractorSettings {
  pub extract_regular_chunks: bool,
  pub extract_transition_lo_x_chunks: bool,
  pub extract_transition_hi_x_chunks: bool,
  pub extract_transition_lo_y_chunks: bool,
  pub extract_transition_hi_y_chunks: bool,
  pub extract_transition_lo_z_chunks: bool,
  pub extract_transition_hi_z_chunks: bool,
}

impl Default for TransvoxelExtractorSettings {
  #[inline]
  fn default() -> Self {
    Self {
      extract_regular_chunks: true,
      extract_transition_lo_x_chunks: true,
      extract_transition_hi_x_chunks: true,
      extract_transition_lo_y_chunks: true,
      extract_transition_hi_y_chunks: true,
      extract_transition_lo_z_chunks: true,
      extract_transition_hi_z_chunks: true,
    }
  }
}

// Extractor

#[derive(Default, Copy, Clone)]
pub struct TransvoxelExtractor<C: ChunkSize> {
  marching_cubes: MarchingCubes<C>,
  transvoxel: Transvoxel<C>,
  settings: TransvoxelExtractorSettings,
}

impl<C: ChunkSize> LodExtractor<C> for TransvoxelExtractor<C> {
  type Chunk = TransvoxelLodChunkMesh;
  type JobInput = TransvoxelJobInput;
  type DependencyKey = ();
  type DependenciesIterator<V: Volume> = TransvoxelJobDependenciesIterator<C, V>;

  #[inline]
  fn create_job<V: Volume>(
    &self,
    aabb: AabbWithSize,
    _neighbor_depths: NeighborDepths,
    volume: V,
    empty_lod_chunk_mesh: Self::Chunk,
  ) -> (Self::JobInput, Self::DependenciesIterator<V>) {
    let input = TransvoxelJobInput { aabb, empty_lod_chunk_mesh };
    let dependencies = TransvoxelJobDependenciesIterator::new(aabb.inner, volume);
    (input, dependencies)
  }

  #[inline]
  fn run_job(
    &self,
    input: Self::JobInput,
    dependency_outputs: &[(Self::DependencyKey, LodJobOutput<MaybeCompressedChunkSampleArray<C>, Self::Chunk>)],
  ) -> Self::Chunk {
    if let (_, LodJobOutput::Sample(chunk_samples)) = &dependency_outputs[0] {
      let TransvoxelJobInput { aabb, empty_lod_chunk_mesh: mut chunk } = input;
      let root_size = aabb.root_size;
      let lores_min = aabb.minimum_point();
      let lores_max = aabb.maximum_point();
      let lores_step = aabb.step::<C>();
      if self.settings.extract_regular_chunks {
        self.marching_cubes.extract_chunk(lores_min, lores_step, &chunk_samples, &mut chunk.regular);
      }
      if lores_step != 1 { // At max LOD level, no need to create transition cells.
        let volume = Sphere::new(SphereSettings::default()); // HACK: create volume here until we port this to use dependencies.
        let hires_step = lores_step / 2;
        if self.settings.extract_transition_lo_x_chunks && lores_min.x > 0 {
          self.extract_transvoxel_chunk(aabb, TransitionSide::LoX, &volume, hires_step, lores_step, &mut chunk.transition_lo_x_chunk);
        }
        if self.settings.extract_transition_hi_x_chunks && lores_max.x < root_size {
          self.extract_transvoxel_chunk(aabb, TransitionSide::HiX, &volume, hires_step, lores_step, &mut chunk.transition_hi_x_chunk);
        }
        if self.settings.extract_transition_lo_y_chunks && lores_min.y > 0 {
          self.extract_transvoxel_chunk(aabb, TransitionSide::LoY, &volume, hires_step, lores_step, &mut chunk.transition_lo_y_chunk);
        }
        if self.settings.extract_transition_hi_y_chunks && lores_max.y < root_size {
          self.extract_transvoxel_chunk(aabb, TransitionSide::HiY, &volume, hires_step, lores_step, &mut chunk.transition_hi_y_chunk);
        }
        if self.settings.extract_transition_lo_z_chunks && lores_min.z > 0 {
          self.extract_transvoxel_chunk(aabb, TransitionSide::LoZ, &volume, hires_step, lores_step, &mut chunk.transition_lo_z_chunk);
        }
        if self.settings.extract_transition_hi_z_chunks && lores_max.z < root_size {
          self.extract_transvoxel_chunk(aabb, TransitionSide::HiZ, &volume, hires_step, lores_step, &mut chunk.transition_hi_z_chunk);
        }
      }
      chunk
    } else {
      panic!("Missing sample dependency output");
    }
  }

  #[inline]
  fn update_render_data(&self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    if self.settings.extract_regular_chunks {
      copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
    }
    if self.settings.extract_transition_lo_x_chunks {
      copy_chunk_vertices(&chunk.transition_lo_x_chunk, vertices, indices, draws);
    }
    if self.settings.extract_transition_hi_x_chunks {
      copy_chunk_vertices(&chunk.transition_hi_x_chunk, vertices, indices, draws);
    }
    if self.settings.extract_transition_lo_y_chunks {
      copy_chunk_vertices(&chunk.transition_lo_y_chunk, vertices, indices, draws);
    }
    if self.settings.extract_transition_hi_y_chunks {
      copy_chunk_vertices(&chunk.transition_hi_y_chunk, vertices, indices, draws);
    }
    if self.settings.extract_transition_lo_z_chunks {
      copy_chunk_vertices(&chunk.transition_lo_z_chunk, vertices, indices, draws);
    }
    if self.settings.extract_transition_hi_z_chunks {
      copy_chunk_vertices(&chunk.transition_hi_z_chunk, vertices, indices, draws);
    }
  }
}

impl<C: ChunkSize> TransvoxelExtractor<C> {
  #[inline]
  pub fn new(marching_cubes: MarchingCubes<C>, transvoxel: Transvoxel<C>, settings: TransvoxelExtractorSettings) -> Self {
    Self { marching_cubes, transvoxel, settings }
  }

  #[inline]
  fn extract_transvoxel_chunk<V: Volume>(
    &self,
    aabb: AabbWithSize,
    side: TransitionSide,
    volume: &V,
    hires_step: u32,
    lores_step: u32,
    chunk_vertices: &mut ChunkMesh,
  ) {
    let hires_chunk_mins = side.subdivided_face_of_side_minimums(aabb);
    let hires_chunk_samples = [
      volume.sample_chunk(hires_chunk_mins[0], hires_step),
      volume.sample_chunk(hires_chunk_mins[1], hires_step),
      volume.sample_chunk(hires_chunk_mins[2], hires_step),
      volume.sample_chunk(hires_chunk_mins[3], hires_step),
    ];
    self.transvoxel.extract_chunk(
      side,
      &hires_chunk_mins,
      &hires_chunk_samples,
      hires_step,
      aabb.minimum_point(),
      lores_step,
      chunk_vertices,
    );
  }
}


// Job input

pub struct TransvoxelJobInput {
  aabb: AabbWithSize,
  empty_lod_chunk_mesh: TransvoxelLodChunkMesh,
}


// Job dependencies iterator

pub struct TransvoxelJobDependenciesIterator<C, V> {
  aabb: Aabb,
  volume: Option<V>,
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize, V: Volume> TransvoxelJobDependenciesIterator<C, V> {
  #[inline]
  fn new(aabb: Aabb, volume: V) -> Self { Self { aabb, volume: Some(volume), _chunk_size_phantom: PhantomData::default() } }
}

impl<C: ChunkSize, V: Volume> Iterator for TransvoxelJobDependenciesIterator<C, V> {
  type Item = ((), LodJob<C, V, TransvoxelExtractor<C>>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let volume = self.volume.take();
    if let Some(volume) = volume {
      Some(((), LodJob::new_sample(self.aabb, volume)))
    } else {
      None
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) { match &self.volume { Some(_) => (1, Some(1)), None => (0, Some(0)) } }
}

impl<C: ChunkSize, V: Volume> ExactSizeIterator for TransvoxelJobDependenciesIterator<C, V> {
  #[inline]
  fn len(&self) -> usize { match &self.volume { Some(_) => 1, None => 0 } }
}


// Chunk mesh

#[derive(Default, Clone, Debug)]
pub struct TransvoxelLodChunkMesh {
  pub regular: ChunkMesh,
  pub transition_lo_x_chunk: ChunkMesh,
  pub transition_hi_x_chunk: ChunkMesh,
  pub transition_lo_y_chunk: ChunkMesh,
  pub transition_hi_y_chunk: ChunkMesh,
  pub transition_lo_z_chunk: ChunkMesh,
  pub transition_hi_z_chunk: ChunkMesh,
}

impl TransvoxelLodChunkMesh {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(
    regular: ChunkMesh,
    transition_lo_x_chunk: ChunkMesh,
    transition_hi_x_chunk: ChunkMesh,
    transition_lo_y_chunk: ChunkMesh,
    transition_hi_y_chunk: ChunkMesh,
    transition_lo_z_chunk: ChunkMesh,
    transition_hi_z_chunk: ChunkMesh,
  ) -> Self {
    Self {
      regular,
      transition_lo_x_chunk,
      transition_hi_x_chunk,
      transition_lo_y_chunk,
      transition_hi_y_chunk,
      transition_lo_z_chunk,
      transition_hi_z_chunk,
    }
  }
}

impl LodChunkMesh for TransvoxelLodChunkMesh {
  #[inline]
  fn is_empty(&self) -> bool {
    self.regular.is_empty() &&
      self.transition_lo_x_chunk.is_empty() &&
      self.transition_hi_x_chunk.is_empty() &&
      self.transition_lo_y_chunk.is_empty() &&
      self.transition_hi_y_chunk.is_empty() &&
      self.transition_lo_z_chunk.is_empty() &&
      self.transition_hi_z_chunk.is_empty()
  }

  #[inline]
  fn clear(&mut self) {
    self.regular.clear();
    self.transition_lo_x_chunk.clear();
    self.transition_hi_x_chunk.clear();
    self.transition_lo_y_chunk.clear();
    self.transition_hi_y_chunk.clear();
    self.transition_lo_z_chunk.clear();
    self.transition_hi_z_chunk.clear();
  }
}
