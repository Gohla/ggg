use simdnoise::NoiseBuilder;
use ultraviolet::{UVec3, Vec3};

pub trait DensityFunction {
  fn points_per_axis(&self) -> u32;

  fn density_at(&self, position: &UVec3) -> f32;
}


pub struct Sphere {
  points_per_axis: u32,
  radius: f32,
}

#[derive(Copy, Clone)]
pub struct SphereSettings {
  pub points_per_axis: u32,
}

impl Default for SphereSettings {
  fn default() -> Self {
    Self { points_per_axis: 32 }
  }
}

impl Sphere {
  pub fn new(settings: SphereSettings) -> Self {
    let points_per_axis = settings.points_per_axis;
    let radius = (points_per_axis - 1) as f32;
    Self { points_per_axis, radius }
  }
}

impl DensityFunction for Sphere {
  fn points_per_axis(&self) -> u32 {
    self.points_per_axis
  }

  fn density_at(&self, position: &UVec3) -> f32 {
    // Transform position from 0..points_per_axis to -half_radius..half_radius.
    let position = Vec3::from(*position) - (Vec3::one() * (self.radius / 2.0));
    0.5 - position.mag() / self.radius
  }
}


pub struct Noise {
  points_per_axis: u32,
  noise: Vec<f32>,
}

#[derive(Copy, Clone)]
pub struct NoiseSettings {
  pub points_per_axis: u32,
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
      points_per_axis: 32 * 4,
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
    let points_per_axis = settings.points_per_axis;
    let points_per_axis_usize = points_per_axis as usize;
    let noise = NoiseBuilder::ridge_3d(points_per_axis_usize, points_per_axis_usize, points_per_axis_usize)
      .with_seed(settings.seed)
      .with_lacunarity(settings.lacunarity)
      .with_freq(settings.frequency)
      .with_gain(settings.gain)
      .with_octaves(settings.octaves)
      .generate_scaled(settings.min, settings.max);
    Self { points_per_axis, noise }
  }
}

impl DensityFunction for Noise {
  #[inline]
  fn points_per_axis(&self) -> u32 {
    self.points_per_axis
  }

  #[inline]
  fn density_at(&self, position: &UVec3) -> f32 {
    self.noise[(position.x + (position.y * self.points_per_axis) + (position.z * self.points_per_axis * self.points_per_axis)) as usize]
  }
}

