use ultraviolet::UVec3;

use crate::chunk::array::Array;
use crate::chunk::index::VoxelIndex;
use crate::chunk::shape::Shape;
use crate::chunk::size::ChunkSize;

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
  pub fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    use ChunkSamples::*;
    match self {
      Zero => 0.0,
      Positive => 1.0,
      Negative => -1.0,
      Mixed(array) => array.sample_index(voxel_index)
    }
  }

  #[inline]
  pub fn sample(&self, index: UVec3) -> f32 {
    use ChunkSamples::*;
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ChunkSampleArray<C: ChunkSize> {
  pub array: C::VoxelChunkArray<f32>,
}

impl<C: ChunkSize> ChunkSampleArray<C> {
  #[inline]
  pub fn new(array: C::VoxelChunkArray<f32>) -> Self {
    Self { array }
  }

  #[inline]
  pub fn new_with(default: f32) -> Self {
    Self::new(C::VoxelChunkArray::new(default))
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
    self.array.index(voxel_index)
  }

  #[inline]
  pub fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 {
    self.array.index_mut(voxel_index)
  }

  #[inline]
  pub fn sample(&self, position: UVec3) -> f32 {
    let voxel_index = C::VoxelChunkShape::index_from_pos(position);
    self.sample_index(voxel_index)
  }

  #[inline]
  pub fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    let voxel_index = C::VoxelChunkShape::index_from_pos(position);
    self.sample_index_mut(voxel_index)
  }

  #[inline]
  pub fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    let voxel_index = C::VoxelChunkShape::index_from_xyz(x, y, z);
    self.array.set(voxel_index, sample);
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
