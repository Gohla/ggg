use std::mem::size_of;
use std::ops::{Deref, RangeBounds};

use bytemuck::Pod;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferSize, BufferSlice, BufferUsages, CommandEncoder, COPY_BUFFER_ALIGNMENT, Device, Label, Queue, ShaderStages};
use wgpu::util::StagingBelt;

#[derive(Debug)]
pub struct GfxBuffer {
  buffer: Buffer,
}

impl GfxBuffer {
  /// Returns the backing [wgpu `Buffer`](Buffer).
  #[inline]
  pub fn as_wgpu_buffer(&self) -> &Buffer { &self.buffer }
}

impl Deref for GfxBuffer {
  type Target = Buffer;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.buffer }
}


// Simple buffer creation

impl GfxBuffer {
  /// Create a buffer on `device` with `usage`, a `size`, whether it should be `mapped_at_creation`, and an optional
  /// debugging `label`.
  ///
  /// If `size` is not a multiple of [COPY_BUFFER_ALIGNMENT], it is increased to be a multiple of it. If that size is
  /// `0`, an empty buffer is created.
  #[inline]
  pub fn from_size(device: &Device, usage: BufferUsages, size: BufferAddress, mapped_at_creation: bool, label: Label) -> Self {
    let size = wgpu::util::align_to(size, COPY_BUFFER_ALIGNMENT);
    let buffer = device.create_buffer(&BufferDescriptor { label, size, usage, mapped_at_creation });
    Self { buffer }
  }

  /// Create a buffer on `device` with `bytes` as content, with `usage`, a `min_size`, and optional debugging `label`.
  ///
  /// The size of the buffer is `bytes.len().max(min_size)`. If that size is not a multiple of [COPY_BUFFER_ALIGNMENT],
  /// it is increased to be a multiple of it. If that size is `0`, an empty buffer is created.
  #[inline]
  pub fn from_bytes_min_size<'a>(device: &Device, bytes: &[u8], usage: BufferUsages, min_size: BufferAddress, label: Label<'a>) -> Self {
    let unpadded_size = (bytes.len() as BufferAddress).max(min_size);
    let buffer = if unpadded_size == 0 {
      device.create_buffer(&BufferDescriptor { label, size: 0, usage, mapped_at_creation: false })
    } else {
      let size = wgpu::util::align_to(unpadded_size, COPY_BUFFER_ALIGNMENT);
      let buffer = device.create_buffer(&BufferDescriptor { label, size, usage, mapped_at_creation: true });
      buffer.slice(..).get_mapped_range_mut()[..unpadded_size as usize].copy_from_slice(bytes);
      buffer.unmap();
      buffer
    };
    Self { buffer }
  }

  /// Create a buffer on `device` with `bytes` as content, with `usage` and optional debugging `label`.
  ///
  /// If the size of `bytes` is not a multiple of [COPY_BUFFER_ALIGNMENT], it is increased to be a multiple of it. If
  /// the size is `0`, an empty buffer is created.
  #[inline]
  pub fn from_bytes<'a>(device: &Device, bytes: &[u8], usage: BufferUsages, label: Label<'a>) -> Self {
    Self::from_bytes_min_size(device, bytes, usage, 0, label)
  }

  /// Create a buffer on `device` with `data` as content, with `usage`, a `min_size` in bytes, and optional
  /// debugging `label`.
  ///
  /// The size of the buffer is `(size_of_val(data)).max(min_size)`. If that size is not a multiple of
  /// [COPY_BUFFER_ALIGNMENT], it is increased to be a multiple of it. If that size is `0`, an empty buffer is created.
  #[inline]
  pub fn from_data_min_size<'a, T: Pod>(device: &Device, data: &[T], usage: BufferUsages, min_size: BufferAddress, label: Label<'a>) -> Self {
    let bytes = bytemuck::cast_slice(data);
    Self::from_bytes_min_size(device, bytes, usage, min_size, label)
  }

  /// Create a buffer on `device` with `data` as content, with `usage` and optional debugging `label`.
  ///
  /// If the size in bytes of `data` is not a multiple of [COPY_BUFFER_ALIGNMENT], it is increased to be a multiple of
  /// it. If the size is `0`, an empty buffer is created.
  #[inline]
  pub fn from_data<'a, T: Pod>(device: &Device, data: &[T], usage: BufferUsages, label: Label<'a>) -> Self {
    Self::from_data_min_size(device, data, usage, 0, label)
  }
}


// Buffer builder

pub struct BufferBuilder<'a> {
  descriptor: BufferDescriptor<'a>,
}

impl<'a> Default for BufferBuilder<'a> {
  #[inline]
  fn default() -> Self {
    Self {
      descriptor: BufferDescriptor {
        label: None,
        size: 0,
        usage: BufferUsages::COPY_DST,
        mapped_at_creation: false,
      },
    }
  }
}

impl<'a> BufferBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  /// Set the debug label of the buffer, used by graphics debuggers for identification.
  #[inline]
  pub fn label(mut self, label: &'a str) -> Self {
    self.descriptor.label = Some(label);
    self
  }

  /// Set the size of the buffer in bytes.
  #[inline]
  pub fn size(mut self, size: BufferAddress) -> Self {
    self.descriptor.size = size;
    self
  }

  /// Sets the usages of the buffer. If the buffer is used in any way that isn't specified here, the operation will
  /// panic.
  #[inline]
  pub fn usage(mut self, usage: BufferUsages) -> Self {
    self.descriptor.usage = usage;
    self
  }
  #[inline]
  pub fn static_vertex_usage(self) -> Self { self.usage(BufferUsages::VERTEX) }
  #[inline]
  pub fn vertex_usage(self) -> Self { self.usage(BufferUsages::VERTEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn static_index_usage(self) -> Self { self.usage(BufferUsages::INDEX) }
  #[inline]
  pub fn index_usage(self) -> Self { self.usage(BufferUsages::INDEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn static_uniform_usage(self) -> Self { self.usage(BufferUsages::UNIFORM) }
  #[inline]
  pub fn uniform_usage(self) -> Self { self.usage(BufferUsages::UNIFORM | BufferUsages::COPY_DST) }
  #[inline]
  pub fn static_storage_usage(self) -> Self { self.usage(BufferUsages::STORAGE) }
  #[inline]
  pub fn storage_usage(self) -> Self { self.usage(BufferUsages::STORAGE | BufferUsages::COPY_DST) }

  /// Sets whether the buffer will be mapped immediately after creation. The buffer does not require
  /// [`BufferUsages::MAP_READ`] nor [`BufferUsages::MAP_WRITE`] use, all buffers are allowed to be mapped at creation.
  ///
  /// If set to `true`, [`size`](Self::size) must be a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn mapped_at_creation(mut self, mapped_at_creation: bool) -> Self {
    self.descriptor.mapped_at_creation = mapped_at_creation;
    self
  }
}


// Buffer creation

impl<'a> BufferBuilder<'a> {
  /// Create the buffer on `device` without setting its content.
  #[inline]
  pub fn build(self, device: &Device) -> GfxBuffer {
    let buffer = device.create_buffer(&self.descriptor);
    GfxBuffer { buffer }
  }

  /// Create the buffer on `device` with `bytes` as content.
  ///
  /// The size of the buffer is `bytes.len().max(size)`. If that size is not a multiple of [COPY_BUFFER_ALIGNMENT], it
  /// is increased to be a multiple of it. If that size is `0`, an empty buffer is created.
  ///
  /// Ignores the previously set [`mapped_at_creation`](Self::mapped_at_creation) value.
  #[inline]
  pub fn build_with_bytes(self, device: &Device, bytes: &[u8]) -> GfxBuffer {
    let buffer = GfxBuffer::from_bytes_min_size(device, bytes, self.descriptor.usage, self.descriptor.size, self.descriptor.label);
    buffer
  }

  /// Create the buffer on `device` with `data` as content.
  ///
  /// The size of the buffer is `(data.len() * size_of::<T>()).max(size)`. If that size is not a multiple of
  /// [COPY_BUFFER_ALIGNMENT], it is increased to be a multiple of it. If that size is `0`, an empty buffer is created.
  ///
  /// Ignores the previously set [`mapped_at_creation`](Self::mapped_at_creation) value.
  #[inline]
  pub fn build_with_data<T: Pod>(self, device: &Device, data: &[T]) -> GfxBuffer {
    GfxBuffer::from_data_min_size(device, data, self.descriptor.usage, self.descriptor.size, self.descriptor.label)
  }
}


// Buffer writing

impl GfxBuffer {
  /// Enqueue writing `bytes` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// Starts writing at `offset` of this buffer, with `offset` being interpreted in terms of bytes.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - `bytes` overruns the end of this buffer starting at `offset`.
  /// - `offset` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  /// - The size of `bytes` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn write_bytes(&self, queue: &Queue, bytes: &[u8], offset: BufferAddress) {
    queue.write_buffer(&self.buffer, offset, bytes);
  }

  /// Enqueue writing `bytes` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - The size of `bytes` is larger than the size of this buffer.
  /// - The size of `bytes` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn write_all_bytes(&self, queue: &Queue, bytes: &[u8]) {
    self.write_bytes(queue, bytes, 0);
  }

  /// Enqueue writing `bytes` at `offset` of this buffer, using `staging_belt` to create a staging buffer. The write
  /// occurs when the command `encoder` is submitted.
  ///
  /// Starts writing at `offset` of this buffer, with `offset` being interpreted in terms of bytes.
  ///
  /// If the size of `bytes` is 0, nothing happens.
  ///
  /// In contrast to [enqueue_write_bytes](Self::write_bytes), this method supports writing bytes with a size
  /// that is *not* a multiple of [COPY_BUFFER_ALIGNMENT]. It supports this by first creating a staging buffer with a
  /// size that *is* aligned, copying `bytes` into that buffer, and then scheduling a buffer copy from the staging
  /// buffer into this buffer.
  ///
  /// It is up to the caller of this method to correctly [finish](StagingBelt::finish) and
  /// [recall](StagingBelt::recall) the `staging_belt`.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - `bytes` overruns the end of this buffer starting at `offset`.
  /// - `offset` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  pub fn write_bytes_staging(
    &self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    bytes: &[u8],
    offset: BufferAddress,
  ) {
    let unpadded_size = bytes.len();
    if unpadded_size == 0 { return; }
    let padded_size = wgpu::util::align_to(unpadded_size as BufferAddress, COPY_BUFFER_ALIGNMENT);
    let padded_size = BufferSize::new(padded_size).unwrap();
    let mut buffer_view = staging_belt.write_buffer(encoder, &self.buffer, offset, padded_size, device);
    buffer_view[..unpadded_size].copy_from_slice(bytes);
  }


  /// Enqueue writing `data` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// Starts writing at `offset` of this buffer, with `offset` being interpreted in terms of `&[T]`.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - `data` overruns the end of this buffer starting at `offset`.
  /// - `offset` converted to a byte offset is not a multiple of [COPY_BUFFER_ALIGNMENT].
  /// - The size of `data` in bytes is not a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn write_data<T: Pod>(&self, queue: &Queue, data: &[T], offset: usize) {
    let bytes = bytemuck::cast_slice(data);
    let offset = offset * size_of::<T>();
    self.write_bytes(queue, bytes, offset as BufferAddress)
  }

  /// Enqueue writing `data` into this buffer. The write occurs at the next `queue` [submit](Queue::submit) call.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - The size of `data` is larger than the size of this buffer.
  /// - The size of `data` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn write_all_data<T: Pod>(&self, queue: &Queue, data: &[T]) {
    self.write_data::<T>(queue, data, 0);
  }

  /// Enqueue writing `data` at `offset` of this buffer, using `staging_belt` to create a staging buffer. The write
  /// occurs when the command `encoder` is submitted.
  ///
  /// Starts writing at `offset` of this buffer, with `offset` being interpreted in terms of `&[T]`.
  ///
  /// If the size of `data` is 0, nothing happens.
  ///
  /// In contrast to [enqueue_write_data](Self::write_data), this method supports writing data with a size
  /// that is *not* a multiple of [COPY_BUFFER_ALIGNMENT]. It supports this by first creating a staging buffer with a
  /// size that *is* aligned, copying `data` into that buffer, and then scheduling a buffer copy from the staging
  /// buffer into this buffer.
  ///
  /// It is up to the caller of this method to correctly [finish](StagingBelt::finish) and
  /// [recall](StagingBelt::recall) the `staging_belt`.
  ///
  /// This method fails (and may cause panics) if:
  ///
  /// - `data` overruns the end of this buffer starting at `offset`.
  /// - `offset` is not a multiple of [COPY_BUFFER_ALIGNMENT].
  #[inline]
  pub fn write_data_staging<T: Pod>(
    &self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    data: &[T],
    offset: usize,
  ) {
    let bytes = bytemuck::cast_slice(data);
    let offset = offset * size_of::<T>();
    self.write_bytes_staging(device, encoder, staging_belt, bytes, offset as BufferAddress);
  }
}


// Slicing

impl GfxBuffer {
  /// Slices this buffer between `bounds`. Offsets in `bounds` are interpreted in terms of `&[T]`.
  ///
  /// Use [slice](Buffer::slice) for byte offsets and unbounded ranges.
  #[inline]
  pub fn slice_data<T: Sized>(&self, bounds: impl RangeBounds<usize>) -> BufferSlice {
    let start = bounds.start_bound().map(|o| (*o * size_of::<T>()) as BufferAddress);
    let end = bounds.end_bound().map(|o| (*o * size_of::<T>()) as BufferAddress);
    self.buffer.slice((start, end))
  }
}


// Uniform buffer utilities

impl<'a> GfxBuffer {
  #[inline]
  pub fn create_uniform_binding_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStages,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    };
    let bind = BindGroupEntry {
      binding: binding_index,
      resource: self.buffer.as_entire_binding(),
    };
    (layout, bind)
  }

  #[inline]
  pub fn create_storage_binding_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStages,
    read_only: bool,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    };
    let bind = BindGroupEntry {
      binding: binding_index,
      resource: self.buffer.as_entire_binding(),
    };
    (layout, bind)
  }
}
