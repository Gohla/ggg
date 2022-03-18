use ultraviolet::UVec3;
use voxel_meshing::chunk::{ChunkSampleArray, ChunkSamples};

use crate::C1;

pub struct ChunkManager {
  // Central chunk (step 2), min=(1, 1, 1)
  central_chunk_samples: ChunkSamples<C1>,
  // LoZ chunks (step 1), mins=[(1, 1, 0), (2, 1, 0), (1, 2, 0), (2, 2, 0)]
  loz_chunk_samples: [ChunkSamples<C1>; 4],
}

impl ChunkManager {
  pub fn new() -> Self {
    Self {
      central_chunk_samples: ChunkSamples::Mixed(ChunkSampleArray::new_positive_zeroed()),
      loz_chunk_samples: [
        ChunkSamples::Mixed(ChunkSampleArray::new_positive_zeroed()),
        ChunkSamples::Mixed(ChunkSampleArray::new_positive_zeroed()),
        ChunkSamples::Mixed(ChunkSampleArray::new_positive_zeroed()),
        ChunkSamples::Mixed(ChunkSampleArray::new_positive_zeroed()),
      ],
    }
  }

  pub fn get_central(&self, position: UVec3) -> f32 {
    return self.central_chunk_samples.sample(position);
  }

  pub fn set_central(&mut self, x: u32, y: u32, z: u32, sample: f32) {
    if let ChunkSamples::Mixed(array) = &mut self.central_chunk_samples {
      array.set(x, y, z, sample);
    }
  }

  pub fn set_all_to_central(&mut self, sample: f32) {

  }

  pub fn flip_all_central(&mut self) {

  }
}


