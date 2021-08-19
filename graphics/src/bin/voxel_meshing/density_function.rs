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

impl Sphere {
  pub fn new(points_per_axis: u32) -> Self {
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

impl Noise {
  pub fn new(points_per_axis: u32, noise: Vec<f32>) -> Self {
    Self { points_per_axis, noise }
  }

  pub fn new_ridge(points_per_axis: u32, min: f32, max: f32) -> Self {
    let points_per_axis_usize = points_per_axis as usize;
    let noise = NoiseBuilder::ridge_3d(points_per_axis_usize, points_per_axis_usize, points_per_axis_usize)
      .with_freq(0.05)
      .with_octaves(5)
      .with_gain(2.0)
      .with_seed(1337)
      .with_lacunarity(0.5)
      .generate_scaled(min, max);
    Self::new(points_per_axis, noise)
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

