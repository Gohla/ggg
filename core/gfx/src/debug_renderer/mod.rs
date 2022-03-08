use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupLayout, BufferAddress, Features, PolygonMode, PrimitiveTopology, RenderPass, RenderPipeline, ShaderModule, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};

use crate::{Frame, Gfx, include_shader};
use crate::bind_group::CombinedBindGroupLayoutBuilder;
use crate::buffer::{BufferBuilder, GfxBuffer};
use crate::render_pass::RenderPassBuilder;
use crate::render_pipeline::RenderPipelineBuilder;

pub struct DebugRenderer {
  uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,

  point_list_render_pipeline: Option<DebugRendererPipeline<PointVertex>>,
  line_list_render_pipeline: Option<DebugRendererPipeline<RegularVertex>>,
  line_strip_render_pipeline: Option<DebugRendererPipeline<RegularVertex>>,
}

impl DebugRenderer {
  pub fn request_features() -> Features {
    Features::POLYGON_MODE_POINT | Features::POLYGON_MODE_LINE
  }

  pub fn new(gfx: &Gfx, view_projection: Mat4) -> Self {
    let point_vertex_shader_module = gfx.device.create_shader_module(&include_shader!("debug_renderer/point_vert"));
    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("debug_renderer/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("debug_renderer/frag"));

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Debug uniform buffer")
      .build_with_data(&gfx.device, &[Uniform { model_view_projection: view_projection }]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Debug uniform bind group layout")
      .with_label("Debug uniform bind group")
      .build(&gfx.device);

    let has_polygon_mode_point_feature = gfx.adapter.features().contains(Features::POLYGON_MODE_POINT);
    let point_list_render_pipeline = has_polygon_mode_point_feature.then(|| DebugRendererPipeline::new(
      gfx,
      &point_vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group_layout,
      PrimitiveTopology::PointList,
      PolygonMode::Point,
      "point list",
    ));

    let has_polygon_mode_line_feature = gfx.adapter.features().contains(Features::POLYGON_MODE_LINE);
    let line_list_render_pipeline = has_polygon_mode_line_feature.then(|| DebugRendererPipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group_layout,
      PrimitiveTopology::LineList,
      PolygonMode::Line,
      "line list",
    ));
    let line_strip_render_pipeline = has_polygon_mode_line_feature.then(|| DebugRendererPipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group_layout,
      PrimitiveTopology::LineStrip,
      PolygonMode::Line,
      "line strip",
    ));

    Self {
      uniform_buffer,
      uniform_bind_group,
      point_list_render_pipeline,
      line_list_render_pipeline,
      line_strip_render_pipeline,
    }
  }

  pub fn draw_point(&mut self, point: Vec3, col: Vec4, size: f32) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.vertices.push(PointVertex::new(point, col, size));
      pipeline.upload_new_vertex_buffer = true;
    }
  }

  pub fn draw_line(&mut self, start_pos: Vec3, end_pos: Vec3, start_col: Vec4, end_col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.vertices.push(RegularVertex::new(start_pos, start_col));
      pipeline.vertices.push(RegularVertex::new(end_pos, end_col));
      pipeline.upload_new_vertex_buffer = true;
    }
  }

  pub fn draw_cube(&mut self, min: Vec3, size: f32, col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.vertices.push(RegularVertex::new(min, col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, 0.0, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min, col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min, col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, 0.0, size), col));

      let max = min + Vec3::new(size, size, size);
      pipeline.vertices.push(RegularVertex::new(max, col));
      pipeline.vertices.push(RegularVertex::new(max + Vec3::new(-size, 0.0, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(max, col));
      pipeline.vertices.push(RegularVertex::new(max + Vec3::new(0.0, -size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(max, col));
      pipeline.vertices.push(RegularVertex::new(max + Vec3::new(0.0, 0.0, -size), col));

      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, size, size), col));

      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, 0.0, size), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, 0.0, size), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, 0.0, size), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(0.0, size, size), col));

      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, 0.0, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, size, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, 0.0, 0.0), col));
      pipeline.vertices.push(RegularVertex::new(min + Vec3::new(size, 0.0, size), col));

      pipeline.upload_new_vertex_buffer = true;
    }
  }

  pub fn clear(&mut self) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.vertices.clear();
      pipeline.upload_new_vertex_buffer = true;
    }
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.vertices.clear();
      pipeline.upload_new_vertex_buffer = true;
    }
    if let Some(pipeline) = &mut self.line_strip_render_pipeline {
      pipeline.vertices.clear();
      pipeline.upload_new_vertex_buffer = true;
    }
  }

  pub fn render<'a>(&mut self, gfx: &Gfx, frame: &mut Frame<'a>, model_view_projection: Mat4) {
    if self.line_list_render_pipeline.is_none() && self.line_strip_render_pipeline.is_none() {
      return; // Nothing to do
    }

    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform { model_view_projection }]);

    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.reupload_vertex_buffer_if_needed(gfx);
    }
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.reupload_vertex_buffer_if_needed(gfx);
    }
    if let Some(pipeline) = &mut self.line_strip_render_pipeline {
      pipeline.reupload_vertex_buffer_if_needed(gfx);
    }

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Debug render pass")
      .begin_render_pass_for_swap_chain_with_load(frame.encoder, &frame.output_texture);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.draw(&mut render_pass);
    }
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.draw(&mut render_pass);
    }
    if let Some(pipeline) = &mut self.line_strip_render_pipeline {
      pipeline.draw(&mut render_pass);
    }
  }
}

// Pipeline

struct DebugRendererPipeline<V> {
  render_pipeline: RenderPipeline,
  vertex_buffer: GfxBuffer,
  upload_new_vertex_buffer: bool,
  vertices: Vec<V>,
  label: &'static str,
}

impl<V: Vertex + Pod> DebugRendererPipeline<V> {
  fn new(
    gfx: &Gfx,
    vertex_shader_module: &ShaderModule,
    fragment_shader_module: &ShaderModule,
    uniform_bind_group_layout: &BindGroupLayout,
    primitive_topology: PrimitiveTopology,
    polygon_mode: PolygonMode,
    label: &'static str,
  ) -> Self {
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_premultiplied_alpha_blending_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[V::buffer_layout()])
      .with_primitive_topology(primitive_topology)
      .with_polygon_mode(polygon_mode)
      .with_cull_mode(None)
      .with_layout_label(&format!("Debug {} pipeline layout", label))
      .with_label(&format!("Debug {} render pipeline", label))
      .build(&gfx.device);
    let vertices = Vec::new();
    let vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label(&format!("Debug {} vertex buffer", label))
      .build_with_data(&gfx.device, &vertices);
    Self {
      render_pipeline,
      vertex_buffer,
      upload_new_vertex_buffer: false,
      vertices,
      label,
    }
  }

  fn reupload_vertex_buffer_if_needed(&mut self, gfx: &Gfx) {
    if self.upload_new_vertex_buffer {
      self.vertex_buffer = BufferBuilder::new()
        .with_static_vertex_usage()
        .with_label(&format!("Debug {} vertex buffer", self.label))
        .build_with_data(&gfx.device, &self.vertices);
    }
    self.upload_new_vertex_buffer = false;
  }

  fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
    render_pass.push_debug_group(&format!("Debug draw {}", self.label));
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..self.vertex_buffer.len as u32, 0..1);
    render_pass.pop_debug_group();
  }
}

// Uniform data

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  model_view_projection: Mat4,
}

// Vertex data

trait Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static>;
}

// Point vertex data

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PointVertex {
  pos: Vec3,
  col: Vec4,
  size: f32,
}

impl Vertex for PointVertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
      0 => Float32x3,
      1 => Float32x4,
      2 => Float32
    ];
    VertexBufferLayout {
      array_stride: size_of::<Self>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }
}

impl PointVertex {
  #[inline]
  fn new(pos: Vec3, col: Vec4, size: f32) -> Self {
    Self { pos, col, size }
  }
}

// Regular vertex data

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct RegularVertex {
  pos: Vec3,
  col: Vec4,
}

impl Vertex for RegularVertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
      0 => Float32x3,
      1 => Float32x4,
    ];
    VertexBufferLayout {
      array_stride: size_of::<Self>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }
}

impl RegularVertex {
  #[inline]
  fn new(pos: Vec3, col: Vec4) -> Self {
    Self { pos, col }
  }
}
