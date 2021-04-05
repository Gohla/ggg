use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, Device, Sampler, SamplerDescriptor, ShaderStage};

// Sampler builder creation and modification

pub struct SamplerBuilder<'a> {
  descriptor: SamplerDescriptor<'a>,
}

impl<'a> SamplerBuilder<'a> {
  pub fn new() -> Self { Self { descriptor: SamplerDescriptor::default() } }
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
    shader_visibility: ShaderStage,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>);
}

impl<'a> SamplerBuilderSamplerEx<'a> for Sampler {
  fn create_bind_group_entries(&'a self, binding_index: u32, shader_visibility: ShaderStage) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Sampler {
        comparison: false,
        filtering: true,
      },
      count: None,
    };
    let bind_group = BindGroupEntry {
      binding: binding_index,
      resource: BindingResource::Sampler(self),
    };
    (bind_group_layout, bind_group)
  }
}
