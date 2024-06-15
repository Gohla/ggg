use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, Buffer, BufferAddress, BufferBinding, BufferSize, Device, Sampler, TextureView};

pub mod layout_entry;

// Bind group entry creation

pub struct BindGroupEntryBuilder<'a> {
  entry: BindGroupEntry<'a>,
}

impl<'a> BindGroupEntryBuilder<'a> {
  #[inline]
  pub fn new(resource: BindingResource<'a>) -> Self {
    Self {
      entry: BindGroupEntry {
        binding: 0,
        resource,
      }
    }
  }

  #[inline]
  pub fn new_buffer(buffer: &'a Buffer, offset: BufferAddress, size: Option<BufferSize>) -> Self {
    Self::new(BindingResource::Buffer(BufferBinding { buffer, offset, size }))
  }

  #[inline]
  pub fn new_whole_buffer(buffer: &'a Buffer) -> Self {
    Self::new(BindingResource::Buffer(BufferBinding { buffer, offset: 0, size: None }))
  }

  #[inline]
  pub fn new_sampler(sampler: &'a Sampler) -> Self {
    Self::new(BindingResource::Sampler(sampler))
  }

  #[inline]
  pub fn new_texture_view(texture_view: &'a TextureView) -> Self {
    Self::new(BindingResource::TextureView(texture_view))
  }


  #[inline]
  pub fn binding_index(mut self, binding_index: u32) -> Self {
    self.entry.binding = binding_index;
    self
  }


  #[inline]
  pub fn build(self) -> BindGroupEntry<'a> { self.entry }
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
