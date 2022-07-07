use std::ops::Range;

use ultraviolet::UVec3;

use crate::chunk::array::Array;
use crate::chunk::index::VoxelIndex;
use crate::chunk::shape::Shape;
use crate::chunk::size::ChunkSize;

// Chunk samples traits

pub trait ChunkSamples<C: ChunkSize> {
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32;
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    let voxel_index = C::VoxelChunkShape::index_from_pos(position);
    self.sample_index(voxel_index)
  }
}

pub trait MutableChunkSamples<C: ChunkSize> {
  fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32;
  #[inline]
  fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    let voxel_index = C::VoxelChunkShape::index_from_pos(position);
    self.sample_index_mut(voxel_index)
  }
  #[inline]
  fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    let voxel_index = C::VoxelChunkShape::index_from_xyz(x, y, z);
    *self.sample_index_mut(voxel_index) = sample;
  }
}


// Maybe compressed chunk samples

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MaybeCompressedChunkSamples<CS> {
  /// All sampled values in the chunk are exactly `0.0`.
  Zero,
  /// All sampled values in the chunk are positive (i.e., `f32::is_sign_positive() == true`).
  Positive,
  /// All sampled values in the chunk are negative (i.e., `f32::is_sign_negative() == true`).
  Negative,
  /// Sampled values in the chunk are mixed.
  Mixed(CS),
}

impl<C: ChunkSize, CS: ChunkSamples<C>> ChunkSamples<C> for MaybeCompressedChunkSamples<CS> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    use MaybeCompressedChunkSamples::*;
    match self {
      Zero => 0.0,
      Positive => 1.0,
      Negative => -1.0,
      Mixed(inner) => inner.sample_index(voxel_index)
    }
  }
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    use MaybeCompressedChunkSamples::*;
    match self {
      Zero => 0.0,
      Positive => 1.0,
      Negative => -1.0,
      Mixed(inner) => inner.sample(position)
    }
  }
}

pub type MaybeCompressedChunkSampleArray<C> = MaybeCompressedChunkSamples<ChunkSampleArray<C>>;


// Chunk sample array

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ChunkSampleArray<C: ChunkSize> {
  array: C::VoxelChunkArray<f32>,
}

impl<C: ChunkSize> ChunkSampleArray<C> {
  #[inline]
  pub fn new(array: C::VoxelChunkArray<f32>) -> Self { Self { array } }
  #[inline]
  pub fn new_with(default: f32) -> Self { Self::new(C::VoxelChunkArray::new(default)) }
  #[inline]
  pub fn new_positive_zeroed() -> Self { Self::new_with(0.0) }
  #[inline]
  pub fn new_negative_zeroed() -> Self { Self::new_with(-0.0) }

  #[inline]
  pub fn slice_index(&self, range: Range<VoxelIndex>) -> ChunkSampleArraySlice<C> {
    ChunkSampleArraySlice { array: self, range }
  }

  #[inline]
  pub fn set_all_to(&mut self, sample: f32) {
    for s in self.array[..].iter_mut() {
      *s = sample;
    }
  }
  #[inline]
  pub fn flip_all(&mut self) {
    for s in self.array[..].iter_mut() {
      *s *= -1.0;
    }
  }
}

impl<C: ChunkSize> ChunkSamples<C> for ChunkSampleArray<C> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 { self.array[voxel_index] }
}

impl<C: ChunkSize> MutableChunkSamples<C> for ChunkSampleArray<C> {
  #[inline]
  fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 { &mut self.array[voxel_index] }
}

impl<C: ChunkSize> Default for ChunkSampleArray<C> {
  fn default() -> Self { Self::new_positive_zeroed() }
}


// Chunk sample array slice

pub struct ChunkSampleArraySlice<'a, C: ChunkSize> {
  array: &'a ChunkSampleArray<C>,
  range: Range<VoxelIndex>,
}

impl<'a, C: ChunkSize> ChunkSamples<C> for ChunkSampleArraySlice<'a, C> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 { self.array.sample_index(voxel_index) }
}
// 
// impl<'a, C: ChunkSize> MutableChunkSamples<C> for ChunkSampleArraySlice<'a, C> {
//   #[inline]
//   fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 { self.array.sample_index_mut(voxel_index) }
// }
