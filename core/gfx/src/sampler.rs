use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, Device, FilterMode, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages};

// Sampler builder creation and modification

pub struct SamplerBuilder<'a> {
  descriptor: SamplerDescriptor<'a>,
}

impl<'a> SamplerBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self { descriptor: SamplerDescriptor::default() } }


  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }


  #[inline]
  pub fn with_mag_filter(mut self, mag_filter: FilterMode) -> Self {
    self.descriptor.mag_filter = mag_filter;
    self
  }

  #[inline]
  pub fn with_min_filter(mut self, min_filter: FilterMode) -> Self {
    self.descriptor.min_filter = min_filter;
    self
  }
}

// Sampler creation

impl<'a> SamplerBuilder<'a> {
  pub fn build(self, device: &Device) -> Sampler {
    device.create_sampler(&self.descriptor)
  }
}

// Bind group (layout) entries creation

pub trait SamplerBuilderSamplerEx<'a> {
  fn create_bind_group_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStages,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>);
}

impl<'a> SamplerBuilderSamplerEx<'a> for Sampler {
  fn create_bind_group_entries(&'a self, binding_index: u32, shader_visibility: ShaderStages) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Sampler(SamplerBindingType::Filtering), // TODO: make configurable
      count: None,
    };
    let bind_group = BindGroupEntry {
      binding: binding_index,
      resource: BindingResource::Sampler(self),
    };
    (bind_group_layout, bind_group)
  }
}
