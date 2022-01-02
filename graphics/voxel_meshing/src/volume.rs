use simdnoise::NoiseBuilder;
use ultraviolet::{UVec3, Vec3};

pub trait Volume {
  fn sample(&self, position: &UVec3) -> f32;

  fn bounds(&self) -> (UVec3, UVec3);
}


#[derive(Copy, Clone)]
pub struct Sphere {
  radius: f32,
}

#[derive(Copy, Clone)]
pub struct SphereSettings {
  pub radius: f32,
}

impl Default for SphereSettings {
  fn default() -> Self {
    Self { radius: 256.0 }
  }
}

impl Sphere {
  pub fn new(settings: SphereSettings) -> Self {
    Self { radius: settings.radius }
  }
}

impl Volume for Sphere {
  #[inline]
  fn sample(&self, position: &UVec3) -> f32 {
    // Transform position from 0..n to -half_radius..half_radius.
    let position = Vec3::from(*position) - (Vec3::one() * (self.radius / 2.0));
    0.5 - position.mag() / self.radius
  }

  #[inline]
  fn bounds(&self) -> (UVec3, UVec3) {
    (UVec3::zero(), UVec3::one() * u32::MAX)
  }
}


pub struct Noise {
  max_bound: u32,
  noise: Vec<f32>,
}

#[derive(Copy, Clone)]
pub struct NoiseSettings {
  pub max_bound: u32,
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
      max_bound: 256,
      seed: 1337,
      lacunarity: 0.5,
      frequency: 0.05,
      gain: 2.0,
      octaves: 5,
      min: -1.0,
      max: 1.0,
    }
  }
}

impl Noise {
  pub fn new(settings: NoiseSettings) -> Self {
    let max_bound = settings.max_bound;
    let max_bound_usize = max_bound as usize;
    let noise = NoiseBuilder::ridge_3d(max_bound_usize, max_bound_usize, max_bound_usize)
      .with_seed(settings.seed)
      .with_lacunarity(settings.lacunarity)
      .with_freq(settings.frequency)
      .with_gain(settings.gain)
      .with_octaves(settings.octaves)
      .generate_scaled(settings.min, settings.max);
    Self { max_bound, noise }
  }
}

impl Volume for Noise {
  #[inline]
  fn sample(&self, position: &UVec3) -> f32 {
    let idx = (position.x + (position.y * self.max_bound) + (position.z * self.max_bound * self.max_bound)) as usize;
    self.noise.get(idx).map_or(0.0f32, |v|*v) // HACK: return 0.0 when out of bounds.
  }

  #[inline]
  fn bounds(&self) -> (UVec3, UVec3) {
    (UVec3::zero(), UVec3::one() * self.max_bound)
  }
}

