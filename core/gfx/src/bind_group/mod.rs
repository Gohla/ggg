use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device};

pub mod layout_entry;
pub mod entry;

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

pub struct BindGroupLayoutBuilder<'a> {
  descriptor: BindGroupLayoutDescriptor<'a>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
  #[inline]
  pub fn new() -> Self {
    Self {
      descriptor: BindGroupLayoutDescriptor {
        label: None,
        entries: &[],
      }
    }
  }

  #[inline]
  pub fn with_entries(mut self, entries: &'a [BindGroupLayoutEntry]) -> Self {
    self.descriptor.entries = entries;
    self
  }

  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }

  #[inline]
  pub fn build(self, device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&self.descriptor)
  }
}


// Bind group creation

pub struct BindGroupBuilder<'a> {
  descriptor: BindGroupDescriptor<'a>,
}

impl<'a> BindGroupBuilder<'a> {
  #[inline]
  pub fn new(layout: &'a BindGroupLayout) -> Self {
    Self {
      descriptor: BindGroupDescriptor {
        label: None,
        layout,
        entries: &[],
      }
    }
  }

  #[inline]
  pub fn with_entries(mut self, entries: &'a [BindGroupEntry]) -> Self {
    self.descriptor.entries = entries;
    self
  }

  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }

  #[inline]
  pub fn build(self, device: &Device) -> BindGroup {
    device.create_bind_group(&self.descriptor)
  }
}


// Combined bind group (layout) creation

pub struct CombinedBindGroupLayoutBuilder<'a> {
  layout_label: Option<&'a str>,
  layout_entries: &'a [BindGroupLayoutEntry],
  label: Option<&'a str>,
  entries: &'a [BindGroupEntry<'a>],
}

impl<'a> CombinedBindGroupLayoutBuilder<'a> {
  #[inline]
  pub fn new() -> Self {
    Self {
      layout_label: None,
      layout_entries: &[],
      label: None,
      entries: &[],
    }
  }

  #[inline]
  pub fn with_layout_entries(mut self, entries: &'a [BindGroupLayoutEntry]) -> Self {
    self.layout_entries = entries;
    self
  }

  #[inline]
  pub fn with_layout_label(mut self, label: &'a str) -> Self {
    self.layout_label = Some(label);
    self
  }

  #[inline]
  pub fn with_entries(mut self, entries: &'a [BindGroupEntry]) -> Self {
    self.entries = entries;
    self
  }

  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn build(self, device: &Device) -> (BindGroupLayout, BindGroup) {
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: self.layout_label,
      entries: self.layout_entries,
    });
    let bind = device.create_bind_group(&BindGroupDescriptor {
      label: self.label,
      layout: &layout,
      entries: self.entries,
    });
    (layout, bind)
  }
}
