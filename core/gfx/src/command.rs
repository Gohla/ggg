use wgpu::{CommandEncoder, CommandEncoderDescriptor, Device};

pub trait DeviceCommandEncoderEx {
  fn create_default_command_encoder(&self) -> CommandEncoder;
}

impl DeviceCommandEncoderEx for Device {
  #[inline]
  fn create_default_command_encoder(&self) -> CommandEncoder {
    self.create_command_encoder(&CommandEncoderDescriptor { label: None })
  }
}
