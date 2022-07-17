use std::marker::PhantomData;

use ultraviolet::UVec3;

use crate::chunk::array::{Array, ArrayIndex, ArraySlice, ArraySliceMut};
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

pub trait ChunkSamplesMut<C: ChunkSize>: ChunkSamples<C> {
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

  fn set_all_to(&mut self, sample: f32);
  fn flip_all(&mut self);
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
  pub fn slice<'a, CC: ChunkSize, Idx: ArrayIndex<f32, VoxelIndex, Output=[f32]>>(&'a self, index: Idx) -> ChunkSampleSlice<'a, CC> {
    ChunkSampleSlice::<'a, CC>::new(self.array.slice(index))
  }
  #[inline]
  pub fn slice_mut<'a, CC: ChunkSize, Idx: ArrayIndex<f32, VoxelIndex, Output=[f32]>>(&'a mut self, index: Idx) -> ChunkSampleSliceMut<'a, CC> {
    ChunkSampleSliceMut::<'a, CC>::new(self.array.slice_mut(index))
  }
  #[inline]
  pub fn offset<CC: ChunkSize>(&self, offset: UVec3, step: u32) -> ChunkSampleOffset<C, ChunkSampleArray<C>, CC> {
    ChunkSampleOffset::new(self, offset, step)
  }
  #[inline]
  pub fn offset_mut<CC: ChunkSize>(&mut self, offset: UVec3, step: u32) -> ChunkSampleOffsetMut<C, ChunkSampleArray<C>, CC> {
    ChunkSampleOffsetMut::new(self, offset, step)
  }
}

impl<C: ChunkSize> ChunkSamples<C> for ChunkSampleArray<C> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 { self.array[voxel_index] }
}

impl<C: ChunkSize> ChunkSamplesMut<C> for ChunkSampleArray<C> {
  #[inline]
  fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 { &mut self.array[voxel_index] }

  #[inline]
  fn set_all_to(&mut self, sample: f32) {
    for s in self.array[..].iter_mut() {
      *s = sample;
    }
  }
  #[inline]
  fn flip_all(&mut self) {
    for s in self.array[..].iter_mut() {
      *s *= -1.0;
    }
  }
}

impl<C: ChunkSize> Default for ChunkSampleArray<C> {
  #[inline]
  fn default() -> Self { Self::new_positive_zeroed() }
}


// Chunk sample slice

#[repr(transparent)]
pub struct ChunkSampleSlice<'a, C: ChunkSize> {
  slice: ArraySlice<'a, f32, VoxelIndex>,
  _phantom: PhantomData<C>,
}

impl<'a, C: ChunkSize> ChunkSamples<C> for ChunkSampleSlice<'a, C> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 { self.slice[voxel_index] }
}

impl<'a, C: ChunkSize> ChunkSampleSlice<'a, C> {
  #[inline]
  fn new(slice: ArraySlice<'a, f32, VoxelIndex>) -> Self { Self { slice, _phantom: PhantomData::default() } }
}


// Chunk sample slice mutable

#[repr(transparent)]
pub struct ChunkSampleSliceMut<'a, C: ChunkSize> {
  slice: ArraySliceMut<'a, f32, VoxelIndex>,
  _phantom: PhantomData<C>,
}

impl<'a, C: ChunkSize> ChunkSamples<C> for ChunkSampleSliceMut<'a, C> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 { self.slice[voxel_index] }
}

impl<'a, C: ChunkSize> ChunkSamplesMut<C> for ChunkSampleSliceMut<'a, C> {
  #[inline]
  fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 { &mut self.slice[voxel_index] }

  #[inline]
  fn set_all_to(&mut self, sample: f32) {
    for s in self.slice[..].iter_mut() {
      *s = sample;
    }
  }
  #[inline]
  fn flip_all(&mut self) {
    for s in self.slice[..].iter_mut() {
      *s *= -1.0;
    }
  }
}

impl<'a, C: ChunkSize> ChunkSampleSliceMut<'a, C> {
  #[inline]
  fn new(slice: ArraySliceMut<'a, f32, VoxelIndex>) -> Self { Self { slice, _phantom: PhantomData::default() } }
}


// Chunk sample offset

pub struct ChunkSampleOffset<'a, CS: ChunkSize, S: ChunkSamples<CS>, CT: ChunkSize> {
  samples: &'a S,
  offset: UVec3,
  step: u32,
  _phantom_cs: PhantomData<CS>,
  _phantom_ct: PhantomData<CT>,
}

impl<'a, CS: ChunkSize, S: ChunkSamples<CS>, CT: ChunkSize> ChunkSamples<CT> for ChunkSampleOffset<'a, CS, S, CT> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    let voxel_position = CT::VoxelChunkShape::index_into_pos(voxel_index);
    let position = self.offset + voxel_position * self.step;
    let index = CS::VoxelChunkShape::index_from_pos(position);
    self.samples.sample_index(index)
  }
}

impl<'a, CS: ChunkSize, S: ChunkSamples<CS>, CT: ChunkSize> ChunkSampleOffset<'a, CS, S, CT> {
  #[inline]
  fn new(samples: &'a S, offset: UVec3, step: u32) -> Self {
    Self {
      samples,
      offset,
      step,
      _phantom_cs: PhantomData::default(),
      _phantom_ct: PhantomData::default(),
    }
  }
}


// Chunk sample offset mutable

pub struct ChunkSampleOffsetMut<'a, CS: ChunkSize, S: ChunkSamplesMut<CS>, CT: ChunkSize> {
  samples: &'a mut S,
  offset: UVec3,
  step: u32,
  _phantom_cs: PhantomData<CS>,
  _phantom_ct: PhantomData<CT>,
}

impl<'a, CS: ChunkSize, S: ChunkSamplesMut<CS>, CT: ChunkSize> ChunkSamples<CT> for ChunkSampleOffsetMut<'a, CS, S, CT> {
  #[inline]
  fn sample_index(&self, voxel_index: VoxelIndex) -> f32 {
    self.samples.sample_index(self.offset_index(voxel_index))
  }
}

impl<'a, CS: ChunkSize, S: ChunkSamplesMut<CS>, CT: ChunkSize> ChunkSamplesMut<CT> for ChunkSampleOffsetMut<'a, CS, S, CT> {
  #[inline]
  fn sample_index_mut(&mut self, voxel_index: VoxelIndex) -> &mut f32 {
    self.samples.sample_index_mut(self.offset_index(voxel_index))
  }

  #[inline]
  fn set_all_to(&mut self, sample: f32) {
    for z in 0..CT::VOXELS_IN_CHUNK_ROW {
      for y in 0..CT::VOXELS_IN_CHUNK_ROW {
        for x in 0..CT::VOXELS_IN_CHUNK_ROW {
          *self.sample_mut(UVec3::new(x, y, z)) = sample;
        }
      }
    }
  }
  #[inline]
  fn flip_all(&mut self) {
    for z in 0..CT::VOXELS_IN_CHUNK_ROW {
      for y in 0..CT::VOXELS_IN_CHUNK_ROW {
        for x in 0..CT::VOXELS_IN_CHUNK_ROW {
          *self.sample_mut(UVec3::new(x, y, z)) *= -1.0;
        }
      }
    }
  }
}

impl<'a, CS: ChunkSize, S: ChunkSamplesMut<CS>, CT: ChunkSize> ChunkSampleOffsetMut<'a, CS, S, CT> {
  #[inline]
  fn new(samples: &'a mut S, offset: UVec3, step: u32, ) -> Self {
    Self {
      samples,
      offset,
      step,
      _phantom_cs: PhantomData::default(),
      _phantom_ct: PhantomData::default(),
    }
  }

  #[inline]
  fn offset_index(&self, voxel_index: VoxelIndex) -> VoxelIndex {
    let voxel_position = CT::VoxelChunkShape::index_into_pos(voxel_index);
    let position = self.offset + voxel_position * self.step;
    CS::VoxelChunkShape::index_from_pos(position)
  }
}
