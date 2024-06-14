use wgpu::{Device, RenderPass, ShaderModule};

use crate::include_spirv_shader;
use crate::render_pipeline::RenderPipelineBuilder;

pub struct FullScreenTriangle {
  pub vertex_shader: ShaderModule,
}

impl<'a> FullScreenTriangle {
  pub fn new(device: &Device) -> Self {
    let vertex_shader = device.create_shader_module(include_spirv_shader!("full_screen_triangle/vert"));
    Self { vertex_shader }
  }

  pub fn create_render_pipeline_builder(&'a self) -> RenderPipelineBuilder {
    RenderPipelineBuilder::new().with_vertex_module(&self.vertex_shader)
  }

  pub fn draw(&self, render_pass: &mut RenderPass) {
    render_pass.draw(0..3, 0..1);
  }
}
