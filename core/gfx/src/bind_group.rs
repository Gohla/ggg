use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device};

pub trait BindGroupDeviceEx<'a> {
  fn create_bind_layout_group(
    &self,
    layout_entries: &'a [BindGroupLayoutEntry],
    entries: &'a [BindGroupEntry<'a>],
  ) -> (BindGroupLayout, BindGroup);
}

impl<'a> BindGroupDeviceEx<'a> for Device {
  fn create_bind_layout_group(
    &self,
    layout_entries: &'a [BindGroupLayoutEntry],
    entries: &'a [BindGroupEntry<'a>],
  ) -> (BindGroupLayout, BindGroup) {
    let layout = self.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: layout_entries,
      label: None,
    });
    let bind = self.create_bind_group(&BindGroupDescriptor {
      layout: &layout,
      entries,
      label: None,
    });
    (layout, bind)
  }
}
