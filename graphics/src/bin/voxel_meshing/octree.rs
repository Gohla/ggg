#![allow(dead_code)]

use tracing::trace;
use ultraviolet::UVec3;

use crate::{MarchingCubes, Vertex};
use crate::volume::Volume;

#[derive(Copy, Clone)]
pub struct OctreeSettings {
  pub total_size: u32,
  pub chunk_size: u32,
}

impl OctreeSettings {
  #[inline]
  pub fn check(&self) {
    assert_ne!(self.total_size, 0, "Total size may not be 0");
    assert_ne!(self.chunk_size, 0, "Chunk size may not be 0");
    assert!(self.total_size.is_power_of_two(), "Total size {} must be a power of 2", self.total_size);
    assert!(self.chunk_size.is_power_of_two(), "Chunk size {} must be a power of 2", self.chunk_size);
    assert!(self.chunk_size <= self.total_size, "Chunk size {} must be less than or equal to total size {}", self.chunk_size, self.total_size);
  }
}

impl Default for OctreeSettings {
  fn default() -> Self {
    Self {
      total_size: 256,
      chunk_size: 16,
    }
  }
}

pub struct Octree<V: Volume> {
  total_size: u32,
  chunk_size: u32,
  max_lod_level: u32,
  volume: V,
  marching_cubes: MarchingCubes,
}

impl<V: Volume> Octree<V> {
  pub fn new(settings: OctreeSettings, volume: V, marching_cubes: MarchingCubes) -> Self {
    settings.check();
    let max_lod_level = settings.total_size.log(settings.chunk_size) - 1;
    Self {
      total_size: settings.total_size,
      chunk_size: settings.chunk_size,
      max_lod_level,
      volume,
      marching_cubes,
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u32 { self.max_lod_level }

  pub fn generate_into(&self, lod_level: u32, vertices: &mut Vec<Vertex>) {
    assert!(lod_level <= self.max_lod_level, "LOD level {} must be less than or equal to maximum LOD level {}", lod_level, self.max_lod_level);
    let num_chunks_per_axis = self.chunk_size.pow(lod_level); // Note: to get total number of chunks in 3D, multiply lod_level by 3.
    let step = (self.total_size / self.chunk_size.pow(lod_level + 1)) as usize;
    let chunk_step = step as u32 * self.chunk_size;
    for chunk_x in 0..num_chunks_per_axis {
      for chunk_y in 0..num_chunks_per_axis {
        for chunk_z in 0..num_chunks_per_axis {
          let start = UVec3::new(chunk_x * chunk_step, chunk_y * chunk_step, chunk_z * chunk_step);
          let end = start + UVec3::new(chunk_step, chunk_step, chunk_step);
          trace!("Running marching cubes from {:?} to {:?} with step size {}", start ,end, step);
          self.marching_cubes.generate_into(start, end, step, &self.volume, vertices);
        }
      }
    }
  }
}
