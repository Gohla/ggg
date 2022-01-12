use wgpu::{Adapter, CommandEncoder, Device, Instance, Queue, TextureView};

use common::screen::ScreenSize;
use common::timing::FrameTime;

use crate::surface::GfxSurface;

pub mod prelude;
pub mod surface;
pub mod render_pipeline;
pub mod buffer;
pub mod command;
pub mod render_pass;
pub mod texture;
pub mod sampler;
pub mod bind_group;
pub mod camera;
pub mod texture_def;

pub mod debug_renderer;

#[derive(Debug)]
pub struct Gfx {
  pub instance: Instance,
  pub adapter: Adapter,
  pub device: Device,
  pub queue: Queue,
  pub surface: GfxSurface,
}

impl Gfx {
  pub fn resize_surface(&mut self, size: ScreenSize) {
    self.surface.resize(&self.adapter, &self.device, size);
  }
}

#[derive(Debug)]
pub struct Frame<'a> {
  pub screen_size: ScreenSize,
  pub output_texture: &'a TextureView,
  pub encoder: &'a mut CommandEncoder,
  pub extrapolation: f64,
  pub time: FrameTime,
}

#[macro_export]
macro_rules! shader_file {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/", $p, ".spv") }
}

#[macro_export]
macro_rules! include_shader {
  ($p:expr) => { wgpu::include_spirv!($crate::shader_file!($p)) }
}

#[macro_export]
macro_rules! include_shader_without_validation {
  ($p:expr) => {
    wgpu::ShaderModuleDescriptorSpirV {
      label: Some($crate::shader_file!($p)),
      source: wgpu::util::make_spirv_raw(include_bytes!($crate::shader_file!($p))),
    }
  }
}

#[macro_export]
macro_rules! shader_file_for_bin {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/bin/", env!("CARGO_BIN_NAME"), "/", $p, ".spv") }
}

#[macro_export]
macro_rules! include_shader_for_bin {
  ($p:expr) => { wgpu::include_spirv!($crate::shader_file_for_bin!($p)) }
}

#[macro_export]
macro_rules! include_shader_without_validation_for_bin {
  ($p:expr) => {
    wgpu::ShaderModuleDescriptorSpirV {
      label: Some($crate::shader_file!($p)),
      source: wgpu::util::make_spirv_raw(include_bytes!($crate::shader_file_for_bin!($p))),
    }
  }
}
