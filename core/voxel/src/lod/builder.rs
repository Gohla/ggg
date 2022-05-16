use std::marker::PhantomData;

use ultraviolet::Isometry3;

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
    lod_octmap_settings: LodOctmapSettings,
    transform: Isometry3,
  ) -> SimpleLodRenderDataManager<LodOctmap<V, C, E>> {
    let lod_octmap = LodOctmap::new(lod_octmap_settings, transform, self.volume, self.extractor);
    SimpleLodRenderDataManager::new(lod_octmap)
  }

  pub fn build_boxed(
    self,
    lod_octmap_settings: LodOctmapSettings,
    transform: Isometry3,
  ) -> Box<dyn LodRenderDataManager<C>> {
    Box::new(self.build(lod_octmap_settings, transform))
  }
}
