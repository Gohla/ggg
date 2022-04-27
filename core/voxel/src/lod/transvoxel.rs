use crate::chunk::{ChunkSize, ChunkVertices, Vertex};
use crate::lod::aabb::AABB;
use crate::lod::chunk_vertices::LodChunkVertices;
use crate::lod::extract::LodExtractor;
use crate::lod::render::{copy_chunk_vertices, LodDraw, LodRenderDataUpdater};
use crate::marching_cubes::MarchingCubes;
use crate::transvoxel::side::TransitionSide;
use crate::transvoxel::Transvoxel;
use crate::volume::Volume;

// Extractor

#[derive(Default, Copy, Clone)]
pub struct TransvoxelExtractor<C: ChunkSize> {
  marching_cubes: MarchingCubes<C>,
  transvoxel: Transvoxel<C>,
}

impl<C: ChunkSize> LodExtractor<C> for TransvoxelExtractor<C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; MarchingCubes::<C>::SHARED_INDICES_SIZE]:,
  [u16; Transvoxel::<C>::SHARED_INDICES_SIZE]:,
{
  type Chunk = TransvoxelLodChunkVertices;

  #[inline]
  fn extract<V: Volume>(&self, total_size: u32, aabb: AABB, volume: &V, chunk: &mut Self::Chunk) {
    let lores_min = aabb.min();
    let lores_max = aabb.max();
    let lores_step = aabb.step::<C>();
    let chunk_samples = volume.sample_chunk(lores_min, lores_step);
    self.marching_cubes.extract_chunk(lores_min, lores_step, &chunk_samples, &mut chunk.regular);
    if lores_step != 1 { // At max LOD level, no need to create transition cells.
      let hires_step = lores_step / 2;
      if lores_min.x > 0 {
        self.extract_transvoxel_chunk(aabb, TransitionSide::LoX, volume, hires_step, lores_step, &mut chunk.transition_lo_x_chunk);
      }
      if lores_max.x < total_size {
        self.extract_transvoxel_chunk(aabb, TransitionSide::HiX, volume, hires_step, lores_step, &mut chunk.transition_hi_x_chunk);
      }
      if lores_min.y > 0 {
        self.extract_transvoxel_chunk(aabb, TransitionSide::LoY, volume, hires_step, lores_step, &mut chunk.transition_lo_y_chunk);
      }
      if lores_max.y < total_size {
        self.extract_transvoxel_chunk(aabb, TransitionSide::HiY, volume, hires_step, lores_step, &mut chunk.transition_hi_y_chunk);
      }
      if lores_min.z > 0 {
        self.extract_transvoxel_chunk(aabb, TransitionSide::LoZ, volume, hires_step, lores_step, &mut chunk.transition_lo_z_chunk);
      }
      if lores_max.z < total_size {
        self.extract_transvoxel_chunk(aabb, TransitionSide::HiZ, volume, hires_step, lores_step, &mut chunk.transition_hi_z_chunk);
      }
    }
  }
}

impl<C: ChunkSize> TransvoxelExtractor<C> {
  #[inline]
  pub fn new(marching_cubes: MarchingCubes<C>, transvoxel: Transvoxel<C>) -> Self {
    Self { marching_cubes, transvoxel }
  }

  #[inline]
  fn extract_transvoxel_chunk<V: Volume>(
    &self,
    aabb: AABB,
    side: TransitionSide,
    volume: &V,
    hires_step: u32,
    lores_step: u32,
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:,
    [u16; Transvoxel::<C>::SHARED_INDICES_SIZE]:,
  {
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
      aabb.min(),
      lores_step,
      chunk_vertices,
    );
  }
}

// Renderer updater

#[derive(Default, Copy, Clone, Debug)]
pub struct TransvoxelLodRendererSettings {
  pub render_regular_chunks: bool,
  pub render_transition_lo_x_chunks: bool,
  pub render_transition_hi_x_chunks: bool,
  pub render_transition_lo_y_chunks: bool,
  pub render_transition_hi_y_chunks: bool,
  pub render_transition_lo_z_chunks: bool,
  pub render_transition_hi_z_chunks: bool,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct TransvoxelLodRendererUpdater {
  pub settings: TransvoxelLodRendererSettings,
}

impl LodRenderDataUpdater for TransvoxelLodRendererUpdater {
  type Chunk = TransvoxelLodChunkVertices;

  fn update_chunk(&mut self, chunk: &Self::Chunk, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, draws: &mut Vec<LodDraw>) {
    if self.settings.render_regular_chunks {
      copy_chunk_vertices(&chunk.regular, vertices, indices, draws);
    }
    if self.settings.render_transition_lo_x_chunks {
      copy_chunk_vertices(&chunk.transition_lo_x_chunk, vertices, indices, draws);
    }
    if self.settings.render_transition_hi_x_chunks {
      copy_chunk_vertices(&chunk.transition_hi_x_chunk, vertices, indices, draws);
    }
    if self.settings.render_transition_lo_y_chunks {
      copy_chunk_vertices(&chunk.transition_lo_y_chunk, vertices, indices, draws);
    }
    if self.settings.render_transition_hi_y_chunks {
      copy_chunk_vertices(&chunk.transition_hi_y_chunk, vertices, indices, draws);
    }
    if self.settings.render_transition_lo_z_chunks {
      copy_chunk_vertices(&chunk.transition_lo_z_chunk, vertices, indices, draws);
    }
    if self.settings.render_transition_hi_z_chunks {
      copy_chunk_vertices(&chunk.transition_hi_z_chunk, vertices, indices, draws);
    }
  }
}

impl TransvoxelLodRendererUpdater {
  pub fn new(settings: TransvoxelLodRendererSettings) -> Self {
    Self { settings }
  }
}

// Chunk vertices

#[derive(Default, Clone, Debug)]
pub struct TransvoxelLodChunkVertices {
  pub regular: ChunkVertices,
  pub transition_lo_x_chunk: ChunkVertices,
  pub transition_hi_x_chunk: ChunkVertices,
  pub transition_lo_y_chunk: ChunkVertices,
  pub transition_hi_y_chunk: ChunkVertices,
  pub transition_lo_z_chunk: ChunkVertices,
  pub transition_hi_z_chunk: ChunkVertices,
}

impl TransvoxelLodChunkVertices {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn with_chunk_vertices(
    regular: ChunkVertices,
    transition_lo_x_chunk: ChunkVertices,
    transition_hi_x_chunk: ChunkVertices,
    transition_lo_y_chunk: ChunkVertices,
    transition_hi_y_chunk: ChunkVertices,
    transition_lo_z_chunk: ChunkVertices,
    transition_hi_z_chunk: ChunkVertices,
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

impl LodChunkVertices for TransvoxelLodChunkVertices {
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
