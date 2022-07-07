use ultraviolet::{UVec3, Vec3};

use crate::chunk::array::{Array, SliceMut};
use crate::chunk::sample::{ChunkSampleArray, MaybeCompressedChunkSampleArray, MaybeCompressedChunkSamples};
use crate::chunk::shape::Shape;
use crate::chunk::size::ChunkSize;

// Trait

pub trait Volume: Clone + Send + 'static {
  /// Samples a single position, returning its value.
  fn sample(&self, position: UVec3) -> f32;

  /// Samples an entire chunk, returning a value indicating whether the chunk is all zero, positive, negative, or mixed.
  #[profiling::function]
  fn sample_chunk<C: ChunkSize>(&self, start: UVec3, step: u32) -> MaybeCompressedChunkSampleArray<C> {
    let mut all_zero = true;
    let mut all_positive = true;
    let mut all_negative = true;
    let mut array = C::VoxelChunkArray::new(0.0);
    C::VoxelChunkShape::for_all(|x, y, z, i| {
      let position = start + step * UVec3::new(x, y, z);
      let value = self.sample(position);
      if value != 0.0 { all_zero = false; }
      if value.is_sign_positive() { all_negative = false; } else { all_positive = false; }
      array.set(i, value);
    });
    if all_zero {
      MaybeCompressedChunkSamples::Zero
    } else if all_positive {
      MaybeCompressedChunkSamples::Positive
    } else if all_negative {
      MaybeCompressedChunkSamples::Negative
    } else {
      MaybeCompressedChunkSamples::Mixed(ChunkSampleArray::new(array))
    }
  }
}

// Sphere

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SphereSettings {
  pub radius: f32,
}

impl Default for SphereSettings {
  #[inline]
  fn default() -> Self {
    Self { radius: 4096.0 }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
  radius: f32,
  half_radius_vec: Vec3,
}

impl Sphere {
  #[inline]
  pub fn new(settings: SphereSettings) -> Self {
    Self { radius: settings.radius, half_radius_vec: Vec3::one() * (settings.radius / 2.0) }
  }
}

impl Volume for Sphere {
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    // Transform position from 0..n to -half_radius..half_radius.
    let position = Vec3::from(position) - self.half_radius_vec;
    0.5 - position.mag() / self.radius
  }
}

// Noise

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NoiseSettings {
  pub seed: i32,
  pub lacunarity: f32,
  pub frequency: f32,
  pub gain: f32,
  pub octaves: u8,
}

impl Default for NoiseSettings {
  #[inline]
  fn default() -> Self {
    Self {
      seed: 1337,
      lacunarity: 0.5,
      frequency: 0.002,
      gain: 10.0,
      octaves: 3,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Noise {
  settings: NoiseSettings,
}

impl Noise {
  #[inline]
  pub fn new(settings: NoiseSettings) -> Self {
    Self { settings }
  }
}

impl Volume for Noise {
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    let freq = self.settings.frequency;
    unsafe {
      simdnoise::scalar::fbm_3d(position.x as f32 * freq, position.y as f32 * freq, position.z as f32 * freq, self.settings.lacunarity, self.settings.gain, self.settings.octaves, self.settings.seed)
    }
  }
}

// Plus

#[derive(Copy, Clone, Debug)]
pub struct Plus<V1: Volume, V2: Volume> {
  volume_1: V1,
  volume_2: V2,
}

impl<V1: Volume, V2: Volume> Plus<V1, V2> {
  #[inline]
  pub fn new(volume_1: V1, volume_2: V2) -> Self {
    Self { volume_1, volume_2 }
  }
}

impl<V1: Volume, V2: Volume> Volume for Plus<V1, V2> {
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    self.volume_1.sample(position) + self.volume_2.sample(position)
  }
}
