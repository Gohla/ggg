use std::ops::Deref;

use bytemuck::Pod;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, Device, Queue, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

// Buffer builder

pub struct BufferBuilder<'a> {
  descriptor: BufferDescriptor<'a>,
  len: usize,
}

impl<'a> BufferBuilder<'a> {
  #[inline]
  pub fn new() -> Self {
    Self {
      descriptor: BufferDescriptor {
        label: None,
        size: 0,
        usage: BufferUsages::COPY_DST,
        mapped_at_creation: false,
      },
      len: 0,
    }
  }


  #[inline]
  pub fn with_size(mut self, size: BufferAddress) -> Self {
    self.descriptor.size = size;
    self
  }

  #[inline]
  pub fn with_len(mut self, len: usize) -> Self {
    self.len = len;
    self
  }

  #[inline]
  pub fn with_usage(mut self, usage: BufferUsages) -> Self {
    self.descriptor.usage = usage;
    self
  }

  #[inline]
  pub fn with_static_vertex_usage(self) -> Self { self.with_usage(BufferUsages::VERTEX) }
  #[inline]
  pub fn with_vertex_usage(self) -> Self { self.with_usage(BufferUsages::VERTEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_static_index_usage(self) -> Self { self.with_usage(BufferUsages::INDEX) }
  #[inline]
  pub fn with_index_usage(self) -> Self { self.with_usage(BufferUsages::INDEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_static_uniform_usage(self) -> Self { self.with_usage(BufferUsages::UNIFORM) }
  #[inline]
  pub fn with_uniform_usage(self) -> Self { self.with_usage(BufferUsages::UNIFORM | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_static_storage_usage(self) -> Self { self.with_usage(BufferUsages::STORAGE) }
  #[inline]
  pub fn with_storage_usage(self) -> Self { self.with_usage(BufferUsages::STORAGE | BufferUsages::COPY_DST) }

  #[inline]
  pub fn with_mapped_at_creation(mut self, mapped_at_creation: bool) -> Self {
    self.descriptor.mapped_at_creation = mapped_at_creation;
    self
  }

  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }
}

// Buffer creation

pub struct GfxBuffer {
  pub buffer: Buffer,
  pub size: BufferAddress,
  pub len: usize,
}

impl<'a> BufferBuilder<'a> {
  #[inline]
  pub fn build(self, device: &Device) -> GfxBuffer {
    let buffer = device.create_buffer(&self.descriptor);
    let size = self.descriptor.size;
    let len = self.len;
    GfxBuffer { buffer, size, len }
  }

  /// Ignores the previously set `size`, `len`, and `mapped_at_creation` values.
  #[inline]
  pub fn build_with_data<T: Pod>(self, device: &Device, data: &[T]) -> GfxBuffer {
    let contents: &[u8] = bytemuck::cast_slice(data);
    // TODO: create_buffer_init may adjust the size to include padding for alignment, so the size may not be correct?
    let size = contents.len() as BufferAddress;
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: self.descriptor.label,
      contents,
      usage: self.descriptor.usage,
    });
    let len = data.len();
    GfxBuffer { buffer, size, len }
  }
}

// Buffer writing

impl GfxBuffer {
  /// Bytes must fit within the buffer. Offset must be within the size of the buffer and must not cause an overflow when
  /// writing the data.
  #[inline]
  pub fn write_bytes(&self, queue: &Queue, bytes_offset: BufferAddress, bytes: &[u8]) {
    queue.write_buffer(&self, bytes_offset, bytes);
  }

  /// Bytes must fit within the buffer.
  #[inline]
  pub fn write_whole_bytes(&self, queue: &Queue, bytes: &[u8]) {
    self.write_data(queue, 0, bytes);
  }

  /// Data must fit within the buffer. Offset must be within the size of the buffer and must not cause an overflow when
  /// writing the data.
  #[inline]
  pub fn write_data<T: Pod>(&self, queue: &Queue, bytes_offset: BufferAddress, data: &[T]) {
    queue.write_buffer(&self, bytes_offset, bytemuck::cast_slice(data));
  }

  /// Data must fit within the buffer.
  #[inline]
  pub fn write_whole_data<T: Pod>(&self, queue: &Queue, data: &[T]) {
    self.write_data(queue, 0, data);
  }
}

// Uniform buffer utilities

impl<'a> GfxBuffer {
  #[inline]
  pub fn create_uniform_binding_entries(&'a self, binding_index: u32, shader_visibility: ShaderStages) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
      count: None,
    };
    let bind = BindGroupEntry {
      binding: binding_index,
      resource: self.buffer.as_entire_binding(),
    };
    (layout, bind)
  }

  #[inline]
  pub fn create_storage_binding_entries(&'a self, binding_index: u32, shader_visibility: ShaderStages, read_only: bool) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
      count: None,
    };
    let bind = BindGroupEntry {
      binding: binding_index,
      resource: self.buffer.as_entire_binding(),
    };
    (layout, bind)
  }
}

// Deref implementation

impl Deref for GfxBuffer {
  type Target = Buffer;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.buffer }
}
