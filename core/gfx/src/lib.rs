use wgpu::{Adapter, CommandEncoder, Device, Instance, Queue, TextureFormat, TextureView};

use common::screen::ScreenSize;

use crate::render_pass::SingleRenderPassBuilder;
use crate::render_pipeline::RenderPipelineBuilder;
use crate::surface::GfxSurface;
use crate::texture::{GfxTexture, TextureBuilder};

pub mod prelude;
pub mod surface;
pub mod pipeline_layout;
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

pub mod fmt_math;

pub mod debug_renderer;
pub mod full_screen_triangle;

/// Fully initialized graphics instance: handles and data for rendering graphics.
#[derive(Debug)]
pub struct Gfx {
  // DROP: drop `surface` before other fields. Dropping `device` before `surface` causes a segfault. This should be
  // fixed by https://github.com/gfx-rs/wgpu/pull/5640, but it is not? Only segfaults on some demos like `triangle`,
  // maybe has to do with the fact that they have no depth-stencil texture?
  pub surface: GfxSurface,

  pub instance: Instance,
  pub adapter: Adapter,
  pub device: Device,
  pub queue: Queue,

  pub depth_stencil_texture: Option<GfxTexture>,
  pub multisample_output_texture: Option<GfxTexture>,
  pub sample_count: u32,
}

impl Gfx {
  /// Returns the depth-stencil texture if a depth-stencil texture was set.
  #[inline]
  pub fn depth_stencil_texture(&self) -> Option<&GfxTexture> {
    self.depth_stencil_texture.as_ref()
  }
  /// Returns the depth-stencil texture format if a depth-stencil texture was set.
  #[inline]
  pub fn depth_stencil_format(&self) -> Option<TextureFormat> {
    self.depth_stencil_texture().map(|t| t.format())
  }
  /// Returns the depth-stencil texture view if a depth-stencil texture was set.
  #[inline]
  pub fn depth_stencil_texture_view(&self) -> Option<&TextureView> {
    self.depth_stencil_texture().map(|texture| &texture.view)
  }

  /// Returns the multisample output texture if `sample_count > 1`.
  #[inline]
  pub fn multisample_output_texture(&self) -> Option<&GfxTexture> {
    self.multisample_output_texture.as_ref()
  }
  /// Returns the multisample output texture view if `sample_count > 1`.
  #[inline]
  pub fn multisample_output_texture_view(&self) -> Option<&TextureView> {
    self.multisample_output_texture().map(|texture| &texture.view)
  }

  /// Returns a preconfigured [RenderPipelineBuilder] with:
  /// - [depth_texture](RenderPipelineBuilder::depth_texture) if `depth_stencil_texture` is `Some`.
  /// - [multisample_count](RenderPipelineBuilder::multisample_count) from `sample_count`.
  /// - [surface_fragment_target](RenderPipelineBuilder::surface_fragment_target) from `surface`, which sets the
  ///   fragment target to match the output texture format.
  #[inline]
  pub fn render_pipeline_builder(&self) -> RenderPipelineBuilder {
    self.render_pipeline_builder_without_depth_stencil()
      .depth_texture(self.depth_stencil_format())
  }
  /// Returns a preconfigured [RenderPipelineBuilder] with:
  /// - [multisample_count](RenderPipelineBuilder::multisample_count) from `sample_count`.
  /// - [surface_fragment_target](RenderPipelineBuilder::surface_fragment_target) from `surface`, which sets the
  ///   fragment target to match the output texture format.
  #[inline]
  pub fn render_pipeline_builder_without_depth_stencil(&self) -> RenderPipelineBuilder {
    RenderPipelineBuilder::default()
      .surface_fragment_target(&self.surface)
      .multisample_count(self.sample_count)
  }

  /// Resizes the surface (and thus corresponding swapchain textures), the depth-stencil texture if set, and the
  /// multisample output texture if set.
  pub fn resize(&mut self, viewport: ScreenSize) {
    self.surface.resize(&self.adapter, &self.device, viewport);
    if let Some(depth_texture) = &mut self.depth_stencil_texture {
      *depth_texture = TextureBuilder::new_depth(viewport.physical, depth_texture.format())
        .with_sample_count(self.sample_count)
        .build(&self.device);
    }
    if let Some(multisample_output_texture) = &mut self.multisample_output_texture {
      *multisample_output_texture = TextureBuilder::new_multisampled_framebuffer(&self.surface, self.sample_count)
        .build(&self.device);
    }
  }
}

/// Handles and data for rendering graphics in a single frame.
#[derive(Debug)]
pub struct GfxFrame<'a> {
  /// Convenient reference back to graphics facade.
  pub gfx: &'a Gfx,
  /// Current size of the viewport.
  pub viewport: ScreenSize,
  /// Texture to output pixels to.
  pub output_texture: TextureView,
  /// Primary command encoder for recording GPU operations.
  pub encoder: CommandEncoder,
}

impl<'a> GfxFrame<'a> {
  #[inline]
  pub fn render_pass_builder(&mut self) -> SingleRenderPassBuilder {
    SingleRenderPassBuilder::new(&mut self.encoder)
      .maybe_multisample(&self.output_texture, self.gfx.multisample_output_texture_view())
      .maybe_depth_reverse_z(self.gfx.depth_stencil_texture_view())
  }
  #[inline]
  pub fn render_pass_builder_without_depth_stencil(&mut self) -> SingleRenderPassBuilder {
    SingleRenderPassBuilder::new(&mut self.encoder)
      .maybe_multisample(&self.output_texture, self.gfx.multisample_output_texture_view())
  }
}


// Shader loading macros

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
