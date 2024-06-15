//! Growable buffer: a buffer that grows when needed.
//!
//! A [growable buffer](GrowableBuffer) is backed by a [buffer](GfxBuffer). When writing data into a growable buffer, if
//! that data fits in the backing buffer, it is written to the backing buffer. If not, the backing buffer is replaced
//! with a new buffer that fits the data, and the data is written into that new buffer.
//!
//! Backing buffers are always created with [BufferUsages::COPY_DST], for writing into backing buffers.
//!
//! A growable buffer always creates buffers that fit at least your data, but buffers can be larger:
//!
//! - If a [grow_multiplier](GrowableBufferBuilder::grow_multiplier) is set, the minimum size of a new backing buffer is
//!   multiplied by that number. The multiplier is only applied when the buffer needs to grow.
//! - If the size of a backing buffer would not a multiple of [wgpu::COPY_BUFFER_ALIGNMENT], it is increased to be a
//!   multiple of it.
use std::mem::size_of_val;

use bytemuck::Pod;
use wgpu::{BufferAddress, BufferUsages, CommandEncoder, Device};
use wgpu::util::StagingBelt;

use crate::buffer::GfxBuffer;

pub struct GrowableBufferBuilder<L = &'static str> {
  label: Option<L>,
  usage: BufferUsages,
  grow_multiplier: Option<f64>,
}

impl Default for GrowableBufferBuilder {
  #[inline]
  fn default() -> Self {
    Self { label: None, usage: BufferUsages::COPY_DST, grow_multiplier: None }
  }
}
impl GrowableBufferBuilder {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl<L> GrowableBufferBuilder<L> {
  /// Set the debug `label` for backing buffers, used by graphics debuggers for identification.
  #[inline]
  pub fn label<LL>(self, label: LL) -> GrowableBufferBuilder<LL> {
    GrowableBufferBuilder {
      label: Some(label),
      usage: self.usage,
      grow_multiplier: self.grow_multiplier,
    }
  }

  /// Sets the `usage` for backing buffers, specifying how buffers may be used.
  ///
  /// If a backing buffer is used in any way that is not specified here, the operation will panic.
  #[inline]
  pub fn usage(mut self, usage: BufferUsages) -> Self {
    self.usage = usage;
    self
  }
  #[inline]
  pub fn vertex_usage(self) -> Self { self.usage(BufferUsages::VERTEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn index_usage(self) -> Self { self.usage(BufferUsages::INDEX | BufferUsages::COPY_DST) }
  #[inline]
  pub fn uniform_usage(self) -> Self { self.usage(BufferUsages::UNIFORM | BufferUsages::COPY_DST) }
  #[inline]
  pub fn storage_usage(self) -> Self { self.usage(BufferUsages::STORAGE | BufferUsages::COPY_DST) }

  /// Sets the `grow_multiplier`, which multiplies the minimum size of the buffer when it needs to grow.
  #[inline]
  pub fn grow_multiplier(mut self, grow_multiplier: f64) -> Self {
    self.grow_multiplier = Some(grow_multiplier);
    self
  }
}


pub struct GrowableBuffer<L = &'static str> {
  builder: GrowableBufferBuilder<L>,
  buffer: Option<GfxBuffer>,
}

impl<L: AsRef<str> + Clone> GrowableBufferBuilder<L> {
  /// Create a growable buffer without creating a backing buffer. The backing buffer will be created on the next write.
  #[inline]
  pub fn build(mut self) -> GrowableBuffer<L> {
    self.ensure_copy_dst_usage();
    GrowableBuffer { builder: self, buffer: None }
  }

  /// Create a growable buffer with a backing buffer created on `device` with `bytes` as content.
  #[inline]
  pub fn build_with_bytes(mut self, device: &Device, bytes: &[u8]) -> GrowableBuffer<L> {
    self.ensure_copy_dst_usage();
    let buffer = self.buffer_from_bytes_min_size(device, bytes, 0);
    GrowableBuffer { builder: self, buffer: Some(buffer) }
  }

  /// Create a growable buffer with a backing buffer created on `device` with `data` as content.
  #[inline]
  pub fn build_with_data<T: Pod>(mut self, device: &Device, data: &[T]) -> GrowableBuffer<L> {
    self.ensure_copy_dst_usage();
    let buffer = self.buffer_from_data_min_size(device, data, 0);
    GrowableBuffer { builder: self, buffer: Some(buffer) }
  }
}

impl<L: AsRef<str> + Clone> GrowableBuffer<L> {
  /// Write `bytes` into the backing buffer if it is large enough, or grow the backing buffer if not. Returns a
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
  /// # Grow backing buffer
  ///
  /// Otherwise, grow the backing buffer by create a new buffer on `device` with `bytes` as content.
  pub fn write_bytes(
    &mut self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    bytes: &[u8],
  ) -> &GfxBuffer {
    match self.buffer.as_mut() {
      Some(buffer) if buffer.size() >= (bytes.len() as BufferAddress) => {
        buffer.write_bytes_staging(device, encoder, staging_belt, bytes, 0);
      }
      _ => {
        self.replace_with_bytes(device, bytes);
      }
    }
    self.buffer.as_ref().unwrap()
  }

  /// Write `data` into the backing buffer if it is large enough, or grow the backing buffer if not. Returns a
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
  /// # Grow backing buffer
  ///
  /// Otherwise, grow the backing buffer by create a new buffer on `device` with `data` as content.
  pub fn write_data<T: Pod>(
    &mut self,
    device: &Device,
    encoder: &mut CommandEncoder,
    staging_belt: &mut StagingBelt,
    data: &[T],
  ) -> &GfxBuffer {
    match self.buffer.as_mut() {
      Some(buffer) if buffer.size() >= (size_of_val(data) as BufferAddress) => {
        buffer.write_data_staging(device, encoder, staging_belt, data, 0);
      }
      _ => {
        self.replace_with_data(device, data);
      }
    }
    self.buffer.as_ref().unwrap()
  }


  /// Grow the backing buffer if it is smaller than `min_size`. Returns a reference to the backing buffer.
  pub fn ensure_minimum_size(&mut self, device: &Device, min_size: BufferAddress) -> &GfxBuffer {
    match self.buffer.as_mut() {
      Some(buffer) if buffer.size() >= min_size => {
        // Do nothing, buffer is large enough.
      }
      _ => {
        self.replace_with_size(device, min_size);
      }
    }
    self.buffer.as_ref().unwrap()
  }


  /// Replace the backing buffer with a new buffer on `device` with `data` as content. Returns a reference to the
  /// backing buffer.
  #[inline]
  pub fn replace_with_bytes(&mut self, device: &Device, bytes: &[u8]) -> &GfxBuffer {
    let minimum_size = self.grow_size(bytes.len() as BufferAddress).unwrap_or_default();
    tracing::debug!(label = %self.builder.display_label(), size = bytes.len(), minimum_size, "Recreating buffer");
    let buffer = self.builder.buffer_from_bytes_min_size(device, bytes, minimum_size);
    self.buffer.insert(buffer)
  }

  /// Replace the backing buffer with a new buffer on `device` with `data` as content. Returns a reference to the
  /// backing buffer.
  #[inline]
  pub fn replace_with_data<T: Pod>(&mut self, device: &Device, data: &[T]) -> &GfxBuffer {
    let minimum_size = self.grow_size(size_of_val(data) as BufferAddress).unwrap_or_default();
    tracing::debug!(label = %self.builder.display_label(), size = size_of_val(data), minimum_size, "Recreating buffer");
    let buffer = self.builder.buffer_from_data_min_size(device, data, minimum_size);
    self.buffer.insert(buffer)
  }

  /// Replace the backing buffer with a new buffer on `device` with a `size`. Returns a reference to the backing buffer.
  #[inline]
  pub fn replace_with_size(&mut self, device: &Device, desired_size: BufferAddress) -> &GfxBuffer {
    let size = self.grow_size(desired_size).unwrap_or(desired_size);
    tracing::debug!(label = %self.builder.display_label(), size, minimum_size = desired_size, "Recreating buffer");
    let buffer = self.builder.buffer_from_size(device, size);
    self.buffer.insert(buffer)
  }


  /// Returns `Some(buffer)` if this growable buffer has a backing buffer, otherwise returns `None`.
  #[inline]
  pub fn backing_buffer(&self) -> Option<&GfxBuffer> { self.buffer.as_ref() }
}


// Internals

impl<L: AsRef<str> + Clone> GrowableBuffer<L> {
  #[inline]
  fn grow_size(&self, minimum_size: BufferAddress) -> Option<BufferAddress> {
    if let (Some(grow_multiplier), Some(buffer)) = (self.builder.grow_multiplier, &self.buffer) {
      let multiplied_size = (buffer.size() as f64 * grow_multiplier).round() as BufferAddress;
      Some(multiplied_size.max(minimum_size))
    } else {
      None
    }
  }
}
impl<L> GrowableBufferBuilder<L> {
  #[inline]
  fn ensure_copy_dst_usage(&mut self) {
    self.usage |= BufferUsages::COPY_DST;
  }
}
impl<L: AsRef<str> + Clone> GrowableBufferBuilder<L> {
  #[inline]
  fn buffer_from_bytes_min_size(&self, device: &Device, bytes: &[u8], minimum_size: BufferAddress) -> GfxBuffer {
    GfxBuffer::from_bytes_min_size(device, bytes, self.usage, minimum_size, self.buffer_label())
  }
  #[inline]
  fn buffer_from_data_min_size<T: Pod>(&self, device: &Device, data: &[T], minimum_size: BufferAddress) -> GfxBuffer {
    GfxBuffer::from_data_min_size(device, data, self.usage, minimum_size, self.buffer_label())
  }
  #[inline]
  fn buffer_from_size(&self, device: &Device, size: BufferAddress) -> GfxBuffer {
    GfxBuffer::from_size(device, self.usage, size, false, self.buffer_label())
  }

  #[inline]
  fn buffer_label(&self) -> wgpu::Label {
    self.label.as_ref().map(|l| l.as_ref())
  }
  #[inline]
  fn display_label(&self) -> &str {
    self.buffer_label().unwrap_or("unlabelled")
  }
}
