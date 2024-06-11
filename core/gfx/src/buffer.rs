use std::mem::size_of;
use std::ops::Deref;

use bytemuck::Pod;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

// Buffer builder

pub struct BufferBuilder<'a> {
  descriptor: BufferDescriptor<'a>,
  element_count: usize,
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
      element_count: 0,
    }
  }

  /// Set the debug label of the buffer, used by graphics debuggers for identification.
  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }

  /// Set the size of the buffer in bytes.
  #[inline]
  pub fn with_size(mut self, size: BufferAddress) -> Self {
    self.descriptor.size = size;
    self
  }

  /// Sets the usages of the buffer. If the buffer is used in any way that isn't specified here, the operation will
  /// panic.
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

  /// Sets whether the buffer will be mapped immediately after creation. The buffer does not require
  /// [`BufferUsages::MAP_READ`] nor [`BufferUsages::MAP_WRITE`] use, all buffers are allowed to be mapped at creation.
  ///
  /// If set to `true`, [`size`](Self::with_size) must be a multiple of [`COPY_BUFFER_ALIGNMENT`].
  #[inline]
  pub fn with_mapped_at_creation(mut self, mapped_at_creation: bool) -> Self {
    self.descriptor.mapped_at_creation = mapped_at_creation;
    self
  }

  /// Set the element count of the data in the buffer. Not used by wgpu but passed to [`GfxBuffer`] for use later.
  #[inline]
  pub fn with_element_count(mut self, element_count: usize) -> Self {
    self.element_count = element_count;
    self
  }
}

// Buffer creation

pub struct GfxBuffer {
  pub buffer: Buffer,
  pub size: BufferAddress,
  pub element_count: usize,
}

impl<'a> BufferBuilder<'a> {
  /// Create the buffer on `device` without setting its content.
  #[inline]
  pub fn create(self, device: &Device) -> GfxBuffer {
    let buffer = device.create_buffer(&self.descriptor);
    GfxBuffer { buffer, size: self.descriptor.size, element_count: self.element_count }
  }

  /// Create the buffer on `device`, with `content_data`. Overrides the previously set [`size`](Self::with_size),
  /// [`mapped_at_creation`](Self::with_mapped_at_creation), and [`element_count`](Self::with_element_count) values.
  #[inline]
  pub fn create_with_data<T: Pod>(self, device: &Device, content_data: &[T]) -> GfxBuffer {
    let element_count = content_data.len();
    let contents: &[u8] = bytemuck::cast_slice(content_data);
    self
      .with_element_count(element_count)
      .create_with_bytes(device, contents)
  }

  /// Create the buffer on `device`, with `content`.. Overrides the previously set [`size`](Self::with_size), and
  /// [`mapped_at_creation`](Self::with_mapped_at_creation) values.
  #[inline]
  pub fn create_with_bytes(self, device: &Device, contents: &[u8]) -> GfxBuffer {
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: self.descriptor.label,
      contents,
      usage: self.descriptor.usage,
    });
    // Get the size of the buffer since `create_buffer_init` may adjust the size. This is because the buffer is mapped
    // at creation, and thus the size must be a multiple of `COPY_BUFFER_ALIGNMENT`.
    let size = buffer.size();
    GfxBuffer { buffer, size, element_count: self.element_count }
  }
}

// Buffer writing

impl GfxBuffer {
  /// Enqueue writing `bytes` into this buffer, starting at byte `offset` in this buffer. The write occurs at the next
  /// `queue` [submit](Queue::submit) call.
  ///
  /// This method fails if `bytes` overruns the size of this buffer starting at `offset`.
  #[inline]
  pub fn enqueue_write_bytes(&self, queue: &Queue, bytes: &[u8], offset: BufferAddress) {
    queue.write_buffer(&self, offset, bytes);
  }

  /// Enqueue writing `bytes` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// This method fails if `data` overruns the size of this buffer.
  #[inline]
  pub fn enqueue_write_all_bytes(&self, queue: &Queue, bytes: &[u8]) {
    self.enqueue_write_bytes(queue, bytes, 0);
  }

  /// Enqueue writing `data` into this buffer, starting at slice `offset` in this buffer. The write occurs at the next
  /// `queue` [submit](Queue::submit) call.
  ///
  /// This method fails if `bytes` overruns the size of this buffer starting at `offset`.
  #[inline]
  pub fn enqueue_write_data<T: Pod>(&self, queue: &Queue, data: &[T], offset: usize) {
    let bytes = bytemuck::cast_slice(data);
    let offset = (offset * size_of::<T>()) as BufferAddress;
    self.enqueue_write_bytes(queue, bytes, offset)
  }

  /// Enqueue writing `data` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// This method fails if `data` overruns the size of this buffer.
  #[inline]
  pub fn enqueue_write_all_data<T: Pod>(&self, queue: &Queue, data: &[T]) {
    self.enqueue_write_data::<T>(queue, data, 0);
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

// Slicing

impl GfxBuffer {
  pub fn slice_to_end<T: Sized>(&self, offset: BufferAddress) -> BufferSlice {
    let offset = offset * std::mem::size_of::<T>() as u64;
    self.buffer.slice(offset..)
  }
}

// Deref implementation

impl Deref for GfxBuffer {
  type Target = Buffer;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.buffer }
}
