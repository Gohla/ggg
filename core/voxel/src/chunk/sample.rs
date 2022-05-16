use ultraviolet::UVec3;

use crate::chunk::index::{ChunkIndices, VoxelIndex};
use crate::chunk::size::ChunkSize;
use crate::chunk::size::Sliceable;

// Samples

pub enum ChunkSamples<C: ChunkSize> {
  /// All sampled values in the chunk are exactly `0.0`.
  Zero,
  /// All sampled values in the chunk are positive (i.e., `f32::is_sign_positive() == true`).
  Positive,
  /// All sampled values in the chunk are negative (i.e., `f32::is_sign_negative() == true`).
  Negative,
  /// Sampled values in the chunk are mixed.
  Mixed(ChunkSampleArray<C>),
}

impl<C: ChunkSize> ChunkSamples<C> {
  #[inline]
  pub fn sample(&self, index: UVec3) -> f32 {
    use ChunkSamples::*;
    use self::ChunkSamples::{Negative, Positive, Zero};
    match self {
      Zero => 0.0,
      Positive => 1.0,
      Negative => -1.0,
      Mixed(array) => array.sample(index)
    }
  }
}


// Sample array

#[derive(Copy, Clone, Debug)]
pub struct ChunkSampleArray<C: ChunkSize> {
  pub array: C::VoxelsChunkArray<f32>,
}

impl<C: ChunkSize> ChunkSampleArray<C> {
  #[inline]
  pub fn new(array: C::VoxelsChunkArray<f32>) -> Self {
    Self { array }
  }

  #[inline]
  pub fn new_with(default: f32) -> Self {
    Self::new(C::create_voxel_chunk_array(default))
  }

  #[inline]
  pub fn new_positive_zeroed() -> Self {
    Self::new_with(0.0)
  }

  #[inline]
  pub fn new_negative_zeroed() -> Self {
    Self::new_with(-0.0)
  }

  #[inline]
  pub fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    self.array.slice()[voxel_index.into_usize()]
  }

  #[inline]
  pub fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 {
    &mut self.array.slice_mut()[voxel_index.into_usize()]
  }

  #[inline]
  pub fn sample(&self, position: UVec3) -> f32 {
    self.sample_index(C::voxel_index_from_uvec3(position).into())
  }

  #[inline]
  pub fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    self.sample_index_mut(C::voxel_index_from_uvec3(position).into())
  }

  #[inline]
  pub fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    *self.sample_index_mut(C::voxel_index_from_xyz(x, y, z).into()) = sample;
  }

  #[inline]
  pub fn set_all_to(&mut self, sample: f32) {
    for s in self.array.slice_mut().iter_mut() {
      *s = sample;
    }
  }

  #[inline]
  pub fn flip_all(&mut self) {
    for s in self.array.slice_mut().iter_mut() {
      *s *= -1.0;
    }
  }
}


// Default impl

impl<C: ChunkSize> Default for ChunkSampleArray<C> {
  fn default() -> Self {
    Self::new_positive_zeroed()
  }
}
