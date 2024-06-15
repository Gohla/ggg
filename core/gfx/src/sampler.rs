use std::ops::Deref;

use wgpu::{BindGroupEntry, BindGroupLayoutEntry, Device, FilterMode, Sampler, SamplerDescriptor, ShaderStages};
use crate::bind_group::CombinedBinding;

use crate::bind_group::entry::BindGroupEntryBuilder;
use crate::bind_group::layout_entry::{BindGroupLayoutEntryBuilder, SamplerLayoutBuilder};

// Sampler builder creation and modification

#[derive(Default, Clone, Debug)]
pub struct SamplerBuilder<'a> {
  descriptor: SamplerDescriptor<'a>,
}

impl<'a> SamplerBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }


  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }

  #[inline]
  pub fn mag_filter(mut self, mag_filter: FilterMode) -> Self {
    self.descriptor.mag_filter = mag_filter;
    self
  }

  #[inline]
  pub fn min_filter(mut self, min_filter: FilterMode) -> Self {
    self.descriptor.min_filter = min_filter;
    self
  }
}

// Sampler creation

pub struct GfxSampler {
  sampler: Sampler,
}
impl Deref for GfxSampler {
  type Target = Sampler;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.sampler }
}

impl<'a> SamplerBuilder<'a> {
  pub fn build(self, device: &Device) -> GfxSampler {
    let sampler = device.create_sampler(&self.descriptor);
    GfxSampler { sampler }
  }
}

// Bind group (layout) entries creation

impl GfxSampler {
  #[inline]
  pub fn layout_builder(&self) -> BindGroupLayoutEntryBuilder<SamplerLayoutBuilder> {
    BindGroupLayoutEntryBuilder::default().sampler()
  }
  #[inline]
  pub fn layout(&self, binding: u32, visibility: ShaderStages) -> BindGroupLayoutEntry {
    self.layout_builder()
      .binding(binding)
      .visibility(visibility)
      .build()
  }

  #[inline]
  pub fn entry(&self, binding: u32) -> BindGroupEntry {
    BindGroupEntryBuilder::default()
      .binding(binding)
      .sampler(&self.sampler)
      .build()
  }

  #[inline]
  pub fn binding(&self, binding: u32, visibility: ShaderStages) -> CombinedBinding {
    let layout = self.layout(binding, visibility);
    let entry = self.entry(binding);
    CombinedBinding::new(layout, entry)
  }
}
