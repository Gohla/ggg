use wgpu::{Device, RenderPass, ShaderModule};

use crate::{Gfx, include_spirv_shader};
use crate::render_pipeline::RenderPipelineBuilder;

pub struct FullScreenTriangle {
  pub vertex_shader: ShaderModule,
}

impl FullScreenTriangle {
  #[inline]
  pub fn new(device: &Device) -> Self {
    let vertex_shader = device.create_shader_module(include_spirv_shader!("full_screen_triangle/vert"));
    Self { vertex_shader }
  }

  #[inline]
  pub fn configure_render_pipeline_builder<'a>(&'a self, builder: RenderPipelineBuilder<'a>) -> RenderPipelineBuilder<'a> {
    builder.vertex_module(&self.vertex_shader)
  }
  #[inline]
  pub fn create_render_pipeline_builder<'a>(&'a self, gfx: &'a Gfx) -> RenderPipelineBuilder<'a> {
    // Opt out of depth stencil by default, since full screen rendering almost never needs it.
    let builder = gfx.render_pipeline_builder_without_depth_stencil();
    self.configure_render_pipeline_builder(builder)
  }

  #[inline]
  pub fn draw(&self, pass: &mut RenderPass) {
    pass.draw(0..3, 0..1);
  }
}
