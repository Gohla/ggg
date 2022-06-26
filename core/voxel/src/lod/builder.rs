use std::marker::PhantomData;

use ultraviolet::{Isometry3, Mat4};

use gfx::Gfx;

use crate::chunk::size::ChunkSize;
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::{LodOctmap, LodOctmapSettings};
use crate::lod::render::{LodRenderDataManager, SimpleLodRenderDataManager};
use crate::volume::Volume;

pub struct LodManagerBuilder<C, V, E> {
  chunk_size: PhantomData<C>,
  volume: V,
  extractor: E,
}

impl LodManagerBuilder<(), (), ()> {
  pub fn new<C: ChunkSize>() -> LodManagerBuilder<C, (), ()> {
    LodManagerBuilder { chunk_size: PhantomData::default(), volume: (), extractor: () }
  }
}

impl<C: ChunkSize, V, E> LodManagerBuilder<C, V, E> {
  pub fn with_volume<VV: Volume>(self, volume: VV) -> LodManagerBuilder<C, VV, E> {
    LodManagerBuilder { chunk_size: self.chunk_size, volume, extractor: self.extractor }
  }

  pub fn with_extractor<EE: LodExtractor<C>>(self, extractor: EE) -> LodManagerBuilder<C, V, EE> {
    LodManagerBuilder { chunk_size: self.chunk_size, volume: self.volume, extractor }
  }
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> LodManagerBuilder<C, V, E> {
  pub fn build(
    self,
    gfx: &Gfx,
    lod_octmap_settings: LodOctmapSettings,
    transform: Isometry3,
    view_projection_matrix: Mat4,
  ) -> SimpleLodRenderDataManager<LodOctmap<C, V, E>> {
    let lod_octmap = LodOctmap::new(lod_octmap_settings, transform, self.volume, self.extractor);
    SimpleLodRenderDataManager::new(gfx, lod_octmap, view_projection_matrix)
  }

  pub fn build_boxed(
    self,
    gfx: &Gfx,
    lod_octmap_settings: LodOctmapSettings,
    transform: Isometry3,
    view_projection_matrix: Mat4,
  ) -> Box<dyn LodRenderDataManager<C>> {
    Box::new(self.build(gfx, lod_octmap_settings, transform, view_projection_matrix))
  }
}
