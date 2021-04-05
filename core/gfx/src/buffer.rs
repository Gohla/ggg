use std::ops::Deref;

use bytemuck::Pod;
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsage, Device, Queue, ShaderStage};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::bind_group::BindGroupDeviceEx;

pub trait DeviceBufferEx {
  fn create_buffer<T: Pod>(&self, data: &[T], usage: BufferUsage) -> Buffer;

  #[inline]
  fn create_static_vertex_buffer<T: Pod>(&self, data: &[T]) -> Buffer {
    self.create_buffer(data, BufferUsage::VERTEX)
  }

  #[inline]
  fn create_static_index_buffer<T: Pod>(&self, data: &[T]) -> Buffer {
    self.create_buffer(data, BufferUsage::INDEX)
  }

  #[inline]
  fn create_uniform_buffer<T: Pod>(&self, data: &[T]) -> UniformBuffer {
    UniformBuffer::new(self.create_buffer(data, BufferUsage::UNIFORM | BufferUsage::COPY_DST))
  }

  #[inline]
  fn create_static_uniform_buffer<T: Pod>(&self, data: &[T]) -> UniformBuffer {
    UniformBuffer::new(self.create_buffer(data, BufferUsage::UNIFORM))
  }
}

impl DeviceBufferEx for Device {
  #[inline]
  fn create_buffer<T: Pod>(&self, data: &[T], usage: BufferUsage) -> Buffer {
    self.create_buffer_init(&BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(data), usage })
  }
}


pub trait BufferEx {
  fn write_with_offset<T: Pod>(&self, queue: &Queue, offset: BufferAddress, data: &[T]);

  fn write<T: Pod>(&self, queue: &Queue, data: &[T]) {
    self.write_with_offset(queue, 0, data);
  }
}

impl BufferEx for Buffer {
  fn write_with_offset<T: Pod>(&self, queue: &Queue, offset: BufferAddress, data: &[T]) {
    queue.write_buffer(&self, offset, bytemuck::cast_slice(data));
  }
}


pub struct UniformBuffer {
  inner: Buffer,
}

impl<'a> UniformBuffer {
  fn new(buffer: Buffer) -> Self { Self { inner: buffer } }

  pub fn create_binding_entries(&'a self, binding_index: u32, shader_visibility: ShaderStage) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
      count: None,
    };
    let bind = BindGroupEntry {
      binding: binding_index,
      resource: self.inner.as_entire_binding(),
    };
    (layout, bind)
  }

  pub fn create_singleton_binding(&self, device: &Device, shader_visibility: ShaderStage) -> (BindGroupLayout, BindGroup) {
    let (layout_entry, bind_entry) = self.create_binding_entries(0, shader_visibility);
    let (layout, bind) = device.create_bind_layout_group(&[layout_entry], &[bind_entry]);
    (layout, bind)
  }

  #[inline]
  pub fn get_inner(&self) -> &Buffer { &self.inner }

  #[inline]
  pub fn into_inner(self) -> Buffer { self.inner }
}


impl Deref for UniformBuffer {
  type Target = Buffer;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.get_inner() }
}
