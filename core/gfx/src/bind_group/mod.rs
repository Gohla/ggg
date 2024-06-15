use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device, Label};

pub mod layout_entry;
pub mod entry;

#[derive(Clone, Debug)]
pub struct CombinedBinding<'a> {
  pub layout: BindGroupLayoutEntry,
  pub entry: BindGroupEntry<'a>,
}
impl<'a> CombinedBinding<'a> {
  #[inline]
  pub fn new(layout: BindGroupLayoutEntry, entry: BindGroupEntry<'a>) -> Self {
    Self { layout, entry }
  }
}


// Bind group layout creation

#[derive(Default, Copy, Clone, Debug)]
pub struct BindGroupLayoutBuilder<'a> {
  label: Label<'a>,
  entries: &'a [BindGroupLayoutEntry],
}
impl<'a> BindGroupLayoutBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn entries(mut self, entries: &'a [BindGroupLayoutEntry]) -> Self {
    self.entries = entries;
    self
  }

  #[inline]
  pub fn build(self, device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor { label: self.label, entries: self.entries })
  }
}


// Bind group creation

#[derive(Default, Copy, Clone, Debug)]
pub struct BindGroupBuilder<'a> {
  label: Label<'a>,
  entries: &'a [BindGroupEntry<'a>],
}
impl<'a> BindGroupBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn entries(mut self, entries: &'a [BindGroupEntry]) -> Self {
    self.entries = entries;
    self
  }

  #[inline]
  pub fn build(self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor { label: self.label, layout, entries: self.entries })
  }
}


// Combined bind group (layout) creation

#[derive(Default, Copy, Clone, Debug)]
pub struct CombinedBindGroupBuilder<'a> {
  layout: BindGroupLayoutBuilder<'a>,
  entry: BindGroupBuilder<'a>,
}
impl<'a> CombinedBindGroupBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn layout_label(mut self, label: &'a str) -> Self {
    self.layout = self.layout.label(label);
    self
  }

  #[inline]
  pub fn layout_entries(mut self, entries: &'a [BindGroupLayoutEntry]) -> Self {
    self.layout = self.layout.entries(entries);
    self
  }

  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.entry = self.entry.label(label);
    self
  }

  #[inline]
  pub fn entries(mut self, entries: &'a [BindGroupEntry]) -> Self {
    self.entry = self.entry.entries(entries);
    self
  }
}

#[derive(Debug)]
pub struct CombinedBindGroup {
  pub layout: BindGroupLayout,
  pub entry: BindGroup,
}
impl CombinedBindGroup {
  #[inline]
  pub fn new(layout: BindGroupLayout, entry: BindGroup) -> Self {
    Self { layout, entry }
  }
}
impl<'a> CombinedBindGroupBuilder<'a> {
  #[inline]
  pub fn build(self, device: &Device) -> CombinedBindGroup {
    let layout = self.layout.build(device);
    let entry = self.entry.build(device, &layout);
    CombinedBindGroup::new(layout, entry)
  }
}

