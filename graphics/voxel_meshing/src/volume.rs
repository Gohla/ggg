use ultraviolet::{UVec3, Vec3};

// Trait

pub trait Volume {
  fn sample(&self, position: UVec3) -> f32;
}

// Sphere

#[derive(Copy, Clone, Debug)]
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
}

impl Sphere {
  #[inline]
  pub fn new(settings: SphereSettings) -> Self {
    Self { radius: settings.radius }
  }
}

impl Volume for Sphere {
  #[inline]
  fn sample(&self, position: UVec3) -> f32 {
    // Transform position from 0..n to -half_radius..half_radius.
    let position = Vec3::from(position) - (Vec3::one() * (self.radius / 2.0));
    0.5 - position.mag() / self.radius
  }
}

// Noise

#[derive(Copy, Clone, Debug)]
pub struct NoiseSettings {
  pub size: usize,
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
      size: 17, // Chunk size of 16 + 1 because marching cubes samples on the primal grid.
      seed: 1337,
      lacunarity: 0.5,
      frequency: 0.002,
      gain: 10.0,
      octaves: 3,
    }
  }
}

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
