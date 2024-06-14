use wgpu::{BindGroupLayout, Device, PipelineLayout, PipelineLayoutDescriptor, PushConstantRange};

#[derive(Default, Clone, Debug)]
pub struct PipelineLayoutBuilder<'a> {
  layout: PipelineLayoutDescriptor<'a>,
}

impl<'a> PipelineLayoutBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }


  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.layout.label = Some(label);
    self
  }

  #[inline]
  pub fn bind_group_layouts(mut self, bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
    self.layout.bind_group_layouts = bind_group_layouts;
    self
  }

  #[inline]
  pub fn push_constant_ranges(mut self, push_constant_ranges: &'a [PushConstantRange]) -> Self {
    self.layout.push_constant_ranges = push_constant_ranges;
    self
  }


  #[inline]
  pub fn build(self, device: &Device) -> PipelineLayout {
    device.create_pipeline_layout(&self.layout)
  }
}
