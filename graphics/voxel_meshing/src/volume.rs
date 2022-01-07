use std::collections::HashMap;

use simdnoise::NoiseBuilder;
use tracing::debug;
use ultraviolet::{UVec3, Vec3};

// Trait

pub trait Volume {
  fn load(&mut self, start: UVec3, step: u32);

  fn sample(&self, position: UVec3, start: UVec3, step: u32) -> f32;
}

// Sphere

#[derive(Copy, Clone)]
pub struct SphereSettings {
  pub radius: f32,
}

impl Default for SphereSettings {
  fn default() -> Self {
    Self { radius: 4096.0 }
  }
}

#[derive(Copy, Clone)]
pub struct Sphere {
  radius: f32,
}

impl Sphere {
  pub fn new(settings: SphereSettings) -> Self {
    Self { radius: settings.radius }
  }
}

impl Volume for Sphere {
  #[inline]
  fn load(&mut self, _start: UVec3, _step: u32) { /* Nothing to load*/ }

  #[inline]
  fn sample(&self, position: UVec3, _start: UVec3, _step: u32) -> f32 {
    // Transform position from 0..n to -half_radius..half_radius.
    let position = Vec3::from(position) - (Vec3::one() * (self.radius / 2.0));
    0.5 - position.mag() / self.radius
  }
}

// Noise

#[derive(Copy, Clone)]
pub struct NoiseSettings {
  pub size: usize,
  pub seed: i32,
  pub lacunarity: f32,
  pub frequency: f32,
  pub gain: f32,
  pub octaves: u8,
  pub min: f32,
  pub max: f32,
}

impl Default for NoiseSettings {
  fn default() -> Self {
    Self {
      size: 16,
      seed: 1337,
      lacunarity: 0.5,
      frequency: 0.01,
      gain: 2.0,
      octaves: 5,
      min: -1.0,
      max: 1.0,
    }
  }
}

pub struct Noise {
  settings: NoiseSettings,
  noise: HashMap<(UVec3, u32), Vec<f32>>,
}

impl Noise {
  pub fn new(settings: NoiseSettings) -> Self {
    Self { settings, noise: HashMap::new() }
  }
}

impl Volume for Noise {
  fn load(&mut self, start: UVec3, step: u32) {
    let key = (start, step);
    if !self.noise.contains_key(&key) {
      let size = self.settings.size;
      let noise = NoiseBuilder::ridge_3d_offset(start.x as f32, size, start.y as f32, size, start.z as f32, size)
        .with_seed(self.settings.seed)
        .with_lacunarity(self.settings.lacunarity)
        .with_freq(self.settings.frequency * step as f32)
        .with_gain(self.settings.gain)
        .with_octaves(self.settings.octaves)
        .generate_scaled(self.settings.min, self.settings.max);
      self.noise.insert(key, noise);
    }
  }

  #[inline]
  fn sample(&self, position: UVec3, start: UVec3, step: u32) -> f32 {
    let noise = self.noise.get(&(start, step)).unwrap();
    let size = self.settings.size as u32;
    let position = position / step;
    let idx = (position.x + (position.y * size) + (position.z * size * size)) as usize;
    noise.get(idx).map_or(0.0f32, |v| *v) // HACK: return 0.0 when out of bounds.
  }
}

