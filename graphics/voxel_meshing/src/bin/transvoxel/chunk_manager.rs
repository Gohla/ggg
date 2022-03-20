use ultraviolet::UVec3;

use voxel_meshing::chunk::{ChunkSampleArray, ChunkSamples, ChunkSize, GenericChunkSize};

use crate::C1;

type C4 = GenericChunkSize<4>;

pub struct ChunkManager {
  samples: ChunkSampleArray<C4>,
  // // Central chunk (step 2), min=(1, 1, 1)
  // central_chunk_samples: ChunkSamples<C1>,
  // // LoZ chunks (step 1), mins=[(1, 1, 0), (2, 1, 0), (1, 2, 0), (2, 2, 0)]
  // loz_chunk_samples: [ChunkSamples<C1>; 4],
}

impl ChunkManager {
  pub fn new() -> Self {
    Self {
      samples: ChunkSampleArray::new_positive_zeroed(),
    }
  }
}

pub struct MCChunkManager<'a> {
  chunk_manager: &'a mut ChunkManager,
}

impl ChunkManager {
  #[inline]
  pub fn get_mc_chunk_manager(&mut self) -> MCChunkManager {
    MCChunkManager { chunk_manager: self }
  }
}

impl MCChunkManager<'_> {
  #[inline]
  fn get_modified_position(position: UVec3) -> UVec3 {
    UVec3::one() + position * 2
  }

  #[inline]
  pub fn create_samples(&self) -> ChunkSamples<C1> {
    ChunkSamples::Mixed(self.create_sample_array())
  }

  #[inline]
  pub fn create_sample_array(&self) -> ChunkSampleArray<C1> {
    let mut array = ChunkSampleArray::new_positive_zeroed();
    for z in 0..C1::VOXELS_IN_CHUNK_ROW {
      for y in 0..C1::VOXELS_IN_CHUNK_ROW {
        for x in 0..C1::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          *array.sample_mut(position) = self.chunk_manager.samples.sample(Self::get_modified_position(position));
        }
      }
    }
    array
  }

  #[inline]
  pub fn sample(&self, position: UVec3) -> f32 {
    let position = Self::get_modified_position(position);
    self.chunk_manager.samples.sample(position)
  }

  #[inline]
  pub fn sample_mut(&mut self, position: UVec3) -> &mut f32 {
    let position = Self::get_modified_position(position);
    self.chunk_manager.samples.sample_mut(position)
  }

  #[inline]
  pub fn set(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    *self.sample_mut(UVec3::new(x, y, z)) = sample;
  }

  pub fn set_all_to(&mut self, sample: f32) {
    for z in 0..C1::VOXELS_IN_CHUNK_ROW {
      for y in 0..C1::VOXELS_IN_CHUNK_ROW {
        for x in 0..C1::VOXELS_IN_CHUNK_ROW {
          self.set(x, y, z, sample);
        }
      }
    }
  }

  pub fn flip_all(&mut self) {
    for z in 0..C1::VOXELS_IN_CHUNK_ROW {
      for y in 0..C1::VOXELS_IN_CHUNK_ROW {
        for x in 0..C1::VOXELS_IN_CHUNK_ROW {
          *self.sample_mut(UVec3::new(x, y, z)) *= -1.0;
        }
      }
    }
  }
}


