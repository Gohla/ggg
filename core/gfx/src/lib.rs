use wgpu::{Adapter, CommandEncoder, Device, Instance, Queue, TextureFormat, TextureView};

use common::screen::ScreenSize;

use crate::render_pipeline::RenderPipelineBuilder;
use crate::surface::GfxSurface;
use crate::texture::{GfxTexture, TextureBuilder};

pub mod prelude;
pub mod surface;
pub mod pipeline_layout;
pub mod vertex_state;
pub mod fragment_state;
pub mod render_pipeline;
pub mod buffer;
pub mod growable_buffer;
pub mod command;
pub mod render_pass;
pub mod texture;
pub mod sampler;
pub mod bind_group;
pub mod camera;
pub mod texture_def;

pub mod display_math;

pub mod debug_renderer;
pub mod full_screen_triangle;

/// Graphics facade: graphics handles and information.
#[derive(Debug)]
pub struct Gfx {
  pub instance: Instance,
  pub adapter: Adapter,
  pub device: Device,
  pub queue: Queue,
  pub surface: GfxSurface,

  pub depth_stencil_texture: Option<GfxTexture>,
  pub multisampled_framebuffer: Option<GfxTexture>,
  pub sample_count: u32,
}

impl Gfx {
  #[inline]
  pub fn depth_stencil_format(&self) -> Option<TextureFormat> {
    self.depth_stencil_texture.as_ref().map(|t| t.format)
  }


  #[inline]
  pub fn render_pipeline_builder(&self) -> RenderPipelineBuilder {
    self.render_pipeline_builder_without_depth_stencil()
      .with_depth_from_format(self.depth_stencil_format())
  }
  #[inline]
  pub fn render_pipeline_builder_without_depth_stencil(&self) -> RenderPipelineBuilder {
    RenderPipelineBuilder::default()
      .with_surface_fragment_target(&self.surface)
      .with_multisample_count(self.sample_count)
  }


  pub fn resize_surface(&mut self, size: ScreenSize) {
    self.surface.resize(&self.adapter, &self.device, size);
    if let Some(depth_texture) = &mut self.depth_stencil_texture {
      *depth_texture = TextureBuilder::new_depth(size.physical, depth_texture.format)
        .with_sample_count(self.sample_count)
        .build(&self.device);
    }
    if let Some(multisampled_framebuffer) = &mut self.multisampled_framebuffer {
      *multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&self.surface, self.sample_count)
        .build(&self.device);
    }
  }
}

/// Data and handles for rendering a frame.
#[derive(Debug)]
pub struct Render<'a> {
  /// Current size of the screen/window/viewport.
  pub screen_size: ScreenSize,
  /// Swapchain texture to output pixels to.
  pub output_texture: &'a TextureView,
  /// Primary command encoder for recording GPU operations.
  pub encoder: &'a mut CommandEncoder,
}

#[macro_export]
macro_rules! spirv_shader_file {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/", $p, ".spv") }
}
#[macro_export]
macro_rules! include_spirv_shader {
  ($p:expr) => { wgpu::include_spirv!($crate::spirv_shader_file!($p)) }
}
#[macro_export]
macro_rules! include_spirv_shader_raw {
  ($p:expr) => { wgpu::include_spirv_raw!($crate::spirv_shader_file!($p)) }
}

#[macro_export]
macro_rules! spirv_shader_file_for_bin {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/bin/", env!("CARGO_BIN_NAME"), "/", $p, ".spv") }
}
#[macro_export]
macro_rules! include_spirv_shader_for_bin {
  ($p:expr) => { wgpu::include_spirv!($crate::spirv_shader_file_for_bin!($p)) }
}
#[macro_export]
macro_rules! include_spirv_shader_raw_for_bin {
  ($p:expr) => { wgpu::include_spirv_raw!($crate::spirv_shader_file_for_bin!($p)) }
}
