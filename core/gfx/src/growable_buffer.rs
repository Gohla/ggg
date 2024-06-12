// Buffer builder

use std::mem::size_of_val;

use bytemuck::Pod;
use wgpu::{BufferAddress, BufferUsages, CommandEncoder, Device};
use wgpu::util::StagingBelt;

use crate::buffer::GfxBuffer;

pub struct GrowableBufferBuilder<L = &'static str> {
  label: Option<L>,
  usage: BufferUsages,
}

impl Default for GrowableBufferBuilder {
  #[inline]
  fn default() -> Self {
    Self { label: None, usage: BufferUsages::COPY_DST }
  }
}
impl GrowableBufferBuilder {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl<L> GrowableBufferBuilder<L> {
  /// Set the debug `label` of the buffer, used by graphics debuggers for identification.
  #[inline]
  pub fn with_label<LL>(self, label: LL) -> GrowableBufferBuilder<LL> {
    GrowableBufferBuilder {
      label: Some(label),
      usage: self.usage,
    }
  }

  /// Sets the `usage` of the buffer. If the buffer is used in any way that isn't specified here, the operation will
  /// panic.
  #[inline]
  pub fn with_usage(mut self, usage: BufferUsages) -> Self {
    self.usage = usage;
    self
  }
  #[inline]
  pub fn with_vertex_usage(self) -> Self { self.with_usage(BufferUsages::VERTEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_index_usage(self) -> Self { self.with_usage(BufferUsages::INDEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_uniform_usage(self) -> Self { self.with_usage(BufferUsages::UNIFORM | BufferUsages::COPY_DST) }
  #[inline]
  pub fn with_storage_usage(self) -> Self { self.with_usage(BufferUsages::STORAGE | BufferUsages::COPY_DST) }
}


pub struct GrowableBuffer<L = &'static str> {
  builder: GrowableBufferBuilder<L>,
  backing_buffer: Option<GfxBuffer>,
}

impl<L: AsRef<str> + Clone> GrowableBufferBuilder<L> {
  /// Create a growable buffer without creating a backing buffer. The backing buffer will be created on the next write.
  #[inline]
  pub fn create(self) -> GrowableBuffer<L> {
    GrowableBuffer { builder: self, backing_buffer: None }
  }

  /// Create a growable buffer with a backing buffer created on `device` with `bytes` as content.
  #[inline]
  pub fn create_with_bytes(self, device: &Device, bytes: &[u8]) -> GrowableBuffer<L> {
    let buffer = self.create_buffer_with_bytes(device, bytes);
    GrowableBuffer { builder: self, backing_buffer: Some(buffer) }
  }

  /// Create a growable buffer with a backing buffer created on `device` with `data` as content.
  #[inline]
  pub fn create_with_data<T: Pod>(self, device: &Device, data: &[T]) -> GrowableBuffer<L> {
    let buffer = self.create_buffer_with_data(device, data);
    GrowableBuffer { builder: self, backing_buffer: Some(buffer) }
  }
}

impl<L: AsRef<str> + Clone> GrowableBuffer<L> {
  /// Write `bytes` into the backing buffer if it is large enough, or recreate the backing buffer if not. Returns a
  /// reference to the backing buffer.
  ///
  /// # Write to backing buffer
  ///
  /// If the backing buffer is large enough to hold `bytes`, enqueue writing `bytes` into the backing buffer, using
  /// `staging_belt` to create a staging buffer. The write occurs when the command `encoder` is submitted.
  ///
  /// If the size of `bytes` is 0, nothing is written.
  ///
  /// It is up to the caller of this method to correctly [finish](StagingBelt::finish) and
  /// [recall](StagingBelt::recall) the `staging_belt`.
  ///
  /// # (Re)create backing buffer
  ///
  /// Otherwise, create a new backing buffer on `device` with `bytes` as content.
  pub fn write_bytes(
    &mut self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    bytes: &[u8],
  ) -> &GfxBuffer {
    match self.backing_buffer.as_mut() {
      Some(buffer) if (bytes.len() as BufferAddress) <= buffer.size() => {
        buffer.enqueue_write_bytes_via_staging_belt(device, encoder, staging_belt, bytes, 0);
      }
      _ => {
        self.recreate_with_bytes(device, bytes);
      }
    }
    self.backing_buffer.as_ref().unwrap()
  }

  /// Write `data` into the backing buffer if it is large enough, or recreate the backing buffer if not. Returns a
  /// reference to the backing buffer.
  ///
  /// # Write to backing buffer
  ///
  /// If the backing buffer is large enough to hold `data`, enqueue writing `data` into the backing buffer, using
  /// `staging_belt` to create a staging buffer. The write occurs when the command `encoder` is submitted.
  ///
  /// If the size of `bytes` is 0, nothing is written.
  ///
  /// It is up to the caller of this method to correctly [finish](StagingBelt::finish) and
  /// [recall](StagingBelt::recall) the `staging_belt`.
  ///
  /// # (Re)create backing buffer
  ///
  /// Otherwise, create a new backing buffer on `device` with `data` as content.
  pub fn write_data<T: Pod>(
    &mut self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    data: &[T],
  ) -> &GfxBuffer {
    match self.backing_buffer.as_mut() {
      Some(buffer) if (size_of_val(data) as BufferAddress) <= buffer.size() => {
        buffer.enqueue_write_data_via_staging_belt(device, encoder, staging_belt, data, 0);
      }
      _ => {
        self.recreate_with_data(device, data);
      }
    }
    self.backing_buffer.as_ref().unwrap()
  }


  /// Create a new backing buffer on `device` with `data` as content. Returns a reference to the backing buffer.
  #[inline]
  pub fn recreate_with_bytes(&mut self, device: &Device, bytes: &[u8]) -> &GfxBuffer {
    let buffer = self.builder.create_buffer_with_bytes(device, bytes);
    self.backing_buffer.insert(buffer)
  }

  /// Create a new backing buffer on `device` with `data` as content. Returns a reference to the backing buffer.
  #[inline]
  pub fn recreate_with_data<T: Pod>(&mut self, device: &Device, data: &[T]) -> &GfxBuffer {
    let buffer = self.builder.create_buffer_with_data(device, data);
    self.backing_buffer.insert(buffer)
  }


  /// Returns `Some(backing_buffer)` if a backing buffer was created, otherwise returns `None`.
  #[inline]
  pub fn backing_buffer(&self) -> Option<&GfxBuffer> { self.backing_buffer.as_ref() }
}


// Internals: create buffers

impl<L: AsRef<str> + Clone> GrowableBufferBuilder<L> {
  #[inline]
  fn create_buffer_with_bytes(&self, device: &Device, bytes: &[u8]) -> GfxBuffer {
    GfxBuffer::from_bytes(device, bytes, self.buffer_label(), self.usage)
  }
  #[inline]
  fn create_buffer_with_data<T: Pod>(&self, device: &Device, data: &[T]) -> GfxBuffer {
    GfxBuffer::from_data(device, data, self.buffer_label(), self.usage)
  }
  #[inline]
  fn buffer_label(&self) -> wgpu::Label {
    self.label.as_ref().map(|l| l.as_ref())
  }
}
