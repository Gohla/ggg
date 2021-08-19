use simdnoise::NoiseBuilder;
use ultraviolet::UVec3;

pub trait DensityFunction {
  fn points_per_axis(&self) -> u32;

  fn density_at(&self, position: &UVec3) -> f32;
}

pub struct NoiseDensityFunction {
  points_per_axis: u32,
  noise: Vec<f32>,
}

impl NoiseDensityFunction {
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

impl DensityFunction for NoiseDensityFunction {
  #[inline]
  fn points_per_axis(&self) -> u32 {
    self.points_per_axis
  }

  #[inline]
  fn density_at(&self, position: &UVec3) -> f32 {
    self.noise[(position.x + (position.y * self.points_per_axis) + (position.z * self.points_per_axis * self.points_per_axis)) as usize]
  }
}
