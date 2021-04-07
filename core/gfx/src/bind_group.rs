use std::num::NonZeroU32;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType, BufferSize, Device, ShaderStage, TextureSampleType, TextureViewDimension, Sampler, TextureView};

// Bind group layout entry creation

pub struct BindGroupLayoutEntryBuilder {
  entry: BindGroupLayoutEntry
}

impl BindGroupLayoutEntryBuilder {
  #[inline]
  pub fn new(ty: BindingType) -> Self {
    Self {
      entry: BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStage::NONE,
        ty,
        count: None,
      }
    }
  }

  #[inline]
  pub fn new_uniform_buffer(has_dynamic_offset: bool) -> Self {
    Self::new(BindingType::Buffer {
      ty: BufferBindingType::Uniform,
      has_dynamic_offset,
      min_binding_size: None,
    })
  }

  #[inline]
  pub fn new_storage_buffer(has_dynamic_offset: bool, read_only: bool) -> Self {
    Self::new(BindingType::Buffer {
      ty: BufferBindingType::Storage { read_only },
      has_dynamic_offset,
      min_binding_size: None,
    })
  }

  #[inline]
  pub fn new_sampler(filtering: bool, comparison: bool) -> Self {
    Self::new(BindingType::Sampler { filtering, comparison })
  }

  #[inline]
  pub fn new_texture(sample_type: TextureSampleType, view_dimension: TextureViewDimension, multisampled: bool) -> Self {
    Self::new(BindingType::Texture {
      sample_type,
      view_dimension,
      multisampled,
    })
  }

  #[inline]
  pub fn new_float_2d_texture() -> Self {
    Self::new_texture(TextureSampleType::Float { filterable: true }, TextureViewDimension::D2, false)
  }


  #[inline]
  pub fn with_binding(mut self, binding: u32) -> Self {
    self.entry.binding = binding;
    self
  }

  #[inline]
  pub fn with_shader_visibility(mut self, visibility: ShaderStage) -> Self {
    self.entry.visibility = visibility;
    self
  }
  #[inline]
  pub fn with_vertex_shader_visibility(self) -> Self { self.with_shader_visibility(ShaderStage::VERTEX) }
  #[inline]
  pub fn with_fragment_shader_visibility(self) -> Self { self.with_shader_visibility(ShaderStage::FRAGMENT) }
  #[inline]
  pub fn with_compute_shader_visibility(self) -> Self { self.with_shader_visibility(ShaderStage::COMPUTE) }

  #[inline]
  pub fn with_count(mut self, count: NonZeroU32) -> Self {
    self.entry.count = Some(count);
    self
  }


  #[inline]
  pub fn build(self) -> BindGroupLayoutEntry { self.entry }
}

// Bind group entry creation

pub struct BindGroupEntryBuilder<'a> {
  entry: BindGroupEntry<'a>
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
    Self::new(BindingResource::Buffer { buffer, offset, size })
  }

  #[inline]
  pub fn new_whole_buffer(buffer: &'a Buffer) -> Self {
    Self::new(BindingResource::Buffer { buffer, offset: 0, size: None })
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
  pub fn with_binding(mut self, binding: u32) -> Self {
    self.entry.binding = binding;
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