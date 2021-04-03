use bytemuck::Pod;
use wgpu::{Buffer, BufferUsage, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub trait DeviceBufferEx {
  fn create_buffer<T: Pod>(&self, data: &[T], usage: BufferUsage) -> Buffer;

  #[inline]
  fn create_vertex_buffer<T: Pod>(&self, data: &[T]) -> Buffer {
    self.create_buffer(data, BufferUsage::VERTEX)
  }

  #[inline]
  fn create_index_buffer<T: Pod>(&self, data: &[T]) -> Buffer {
    self.create_buffer(data, BufferUsage::INDEX)
  }
}

impl DeviceBufferEx for Device {
  fn create_buffer<T: Pod>(&self, data: &[T], usage: BufferUsage) -> Buffer {
    self.create_buffer_init(&BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(data), usage })
  }
}
