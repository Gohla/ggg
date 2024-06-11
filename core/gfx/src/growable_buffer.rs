// Buffer builder

use bytemuck::Pod;
use wgpu::{BufferAddress, BufferUsages, Device, Queue};

use crate::buffer::GfxBuffer;

pub struct GrowableBufferBuilder<L = &'static str> {
  label: Option<L>,
  usage: BufferUsages,
}

impl Default for GrowableBufferBuilder {
  fn default() -> Self {
    Self {
      label: None,
      usage: BufferUsages::COPY_DST,
    }
  }
}
impl GrowableBufferBuilder {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }
}
impl<L> GrowableBufferBuilder<L> {
  /// Set the debug label of the buffer, used by graphics debuggers for identification.
  #[inline]
  pub fn with_label<LL>(self, label: LL) -> GrowableBufferBuilder<LL> {
    GrowableBufferBuilder {
      label: Some(label),
      usage: self.usage,
    }
  }

  /// Sets the usages of the buffer. If the buffer is used in any way that isn't specified here, the operation will
  /// panic.
  #[inline]
  pub fn with_usage(mut self, usage: BufferUsages) -> Self {
    self.usage = usage;
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
}


pub struct GrowableBuffer<L = &'static str> {
  label: Option<L>,
  usage: BufferUsages,
  buffer: Option<GfxBuffer>,
}

impl<L: AsRef<str> + Clone> GrowableBufferBuilder<L> {
  #[inline]
  pub fn create(self) -> GrowableBuffer<L> {
    GrowableBuffer {
      label: self.label,
      usage: self.usage,
      buffer: None,
    }
  }
}

impl<L: AsRef<str> + Clone> GrowableBuffer<L> {
  pub fn write_all_bytes(&mut self, device: &Device, queue: &Queue, bytes: &[u8], count: usize) -> &GfxBuffer {
    match self.buffer.as_ref() {
      Some(buffer) if (bytes.len() as BufferAddress) <= buffer.size => {
        buffer.enqueue_write_all_bytes(queue, bytes);
      }
      _ => {
        self.create_with_bytes(device, bytes, count);
      }
    }
    self.buffer.as_ref().unwrap()
  }

  pub fn write_all_data<T: Pod>(&mut self, device: &Device, queue: &Queue, data: &[T]) -> &GfxBuffer {
    let count = data.len();
    let bytes: &[u8] = bytemuck::cast_slice(data);
    self.write_all_bytes(device, queue, bytes, count)
  }


  pub fn create_with_bytes(&mut self, device: &Device, bytes: &[u8], count: usize) -> &GfxBuffer {
    let buffer = GfxBuffer::from_bytes(device, bytes, self.label.as_ref().map(|l| l.as_ref()), self.usage, count);
    self.buffer.insert(buffer)
  }

  pub fn create_with_data<T: Pod>(&mut self, device: &Device, data: &[T]) -> &GfxBuffer {
    let buffer = GfxBuffer::from_data(device, data, self.label.as_ref().map(|l| l.as_ref()), self.usage);
    self.buffer.insert(buffer)
  }
}
