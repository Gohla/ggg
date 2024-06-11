use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupLayout, BufferAddress, Features, IndexFormat, PolygonMode, PrimitiveTopology, RenderPass, RenderPipeline, ShaderModule, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};

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
  line_triangle_list_render_pipeline: Option<DebugRendererPipeline<RegularVertex>>,
}

impl DebugRenderer {
  pub fn request_features() -> Features {
    Features::POLYGON_MODE_POINT | Features::POLYGON_MODE_LINE
  }

  pub fn new(gfx: &Gfx, view_projection: Mat4) -> Self {
    let point_vertex_shader_module = gfx.device.create_shader_module(include_shader!("debug_renderer/point_vert"));
    let vertex_shader_module = gfx.device.create_shader_module(include_shader!("debug_renderer/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_shader!("debug_renderer/frag"));

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Debug uniform buffer")
      .create_with_data(&gfx.device, &[Uniform { model_view_projection: view_projection }]);
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
      gfx.sample_count,
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
      gfx.sample_count,
      PrimitiveTopology::LineList,
      PolygonMode::Line,
      "line list",
    ));
    let line_strip_render_pipeline = has_polygon_mode_line_feature.then(|| DebugRendererPipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group_layout,
      gfx.sample_count,
      PrimitiveTopology::LineStrip,
      PolygonMode::Line,
      "line strip",
    ));
    let line_triangle_list_render_pipeline = has_polygon_mode_line_feature.then(|| DebugRendererPipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group_layout,
      gfx.sample_count,
      PrimitiveTopology::TriangleList,
      PolygonMode::Line,
      "line triangle list",
    ));

    Self {
      uniform_buffer,
      uniform_bind_group,
      point_list_render_pipeline,
      line_list_render_pipeline,
      line_strip_render_pipeline,
      line_triangle_list_render_pipeline,
    }
  }


  pub fn draw_point(&mut self, point: Vec3, col: Vec4, size: f32) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.push_vertex(PointVertex::new(point, col, size));
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_points(&mut self, positions: impl IntoIterator<Item=Vec3>, col: Vec4, size: f32) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| PointVertex::new(pos, col, size)));
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_point_vertices(&mut self, vertices: impl IntoIterator<Item=PointVertex>) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.push_vertices(vertices);
      pipeline.upload_buffers = true;
    }
  }


  pub fn draw_line(&mut self, start_pos: Vec3, end_pos: Vec3, start_col: Vec4, end_col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.push_vertex(RegularVertex::new(start_pos, start_col));
      pipeline.push_vertex(RegularVertex::new(end_pos, end_col));
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_lines(&mut self, positions: impl IntoIterator<Item=Vec3>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)));
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_lines_indexed(&mut self, positions: impl IntoIterator<Item=Vec3>, indices: impl IntoIterator<Item=u32>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.push_vertices_and_indices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)), indices);
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_line_vertices(&mut self, vertices: impl IntoIterator<Item=RegularVertex>) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.push_vertices(vertices);
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_line_vertices_indexed(&mut self, vertices: impl IntoIterator<Item=RegularVertex>, indices: impl IntoIterator<Item=u32>) {
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.push_vertices_and_indices(vertices, indices);
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_cube_lines(&mut self, min: Vec3, size: f32, col: Vec4) {
    self.draw_line_vertices_indexed(
      [
        RegularVertex::new(min, col),
        RegularVertex::new(min + Vec3::unit_x() * size, col),
        RegularVertex::new(min + Vec3::unit_y() * size, col),
        RegularVertex::new(min + Vec3::new(size, size, 0.0), col),
        RegularVertex::new(min + Vec3::unit_z() * size, col),
        RegularVertex::new(min + Vec3::new(size, 0.0, size), col),
        RegularVertex::new(min + Vec3::new(0.0, size, size), col),
        RegularVertex::new(min + Vec3::one() * size, col),
      ],
      [
        0, 1, // X
        2, 3,
        4, 5,
        6, 7,
        0, 2, // Y
        1, 3,
        4, 6,
        5, 7,
        0, 4, // Z
        1, 5,
        2, 6,
        3, 7,
      ],
    );
  }

  pub fn draw_axes_lines(&mut self, pos: Vec3, size: f32) {
    let color_x = Vec4::new(1.0, 0.0, 0.0, 1.0);
    let color_y = Vec4::new(0.0, 1.0, 0.0, 1.0);
    let color_z = Vec4::new(0.0, 0.0, 1.0, 1.0);
    self.draw_line_vertices_indexed(
      [
        RegularVertex::new(pos, color_x),
        RegularVertex::new(pos + Vec3::unit_x() * size, color_x),
        RegularVertex::new(pos, color_y),
        RegularVertex::new(pos + Vec3::unit_y() * size, color_y),
        RegularVertex::new(pos, color_z),
        RegularVertex::new(pos + Vec3::unit_z() * size, color_z),
      ],
      [
        0, 1,
        2, 3,
        4, 5,
      ],
    );
  }


  pub fn draw_triangles_wireframe(&mut self, positions: impl IntoIterator<Item=Vec3>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)));
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_triangles_wireframe_indexed(&mut self, positions: impl IntoIterator<Item=Vec3>, indices: impl IntoIterator<Item=u32>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.push_vertices_and_indices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)), indices);
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_triangle_vertices_wireframe(&mut self, vertices: impl IntoIterator<Item=RegularVertex>) {
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.push_vertices(vertices);
      pipeline.upload_buffers = true;
    }
  }

  pub fn draw_triangle_vertices_wireframe_indexed(&mut self, vertices: impl IntoIterator<Item=RegularVertex>, indices: impl IntoIterator<Item=u32>) {
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.push_vertices_and_indices(vertices, indices);
      pipeline.upload_buffers = true;
    }
  }


  pub fn clear(&mut self) {
    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.vertices.clear();
      pipeline.indices.clear();
      pipeline.upload_buffers = true;
    }
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.vertices.clear();
      pipeline.indices.clear();
      pipeline.upload_buffers = true;
    }
    if let Some(pipeline) = &mut self.line_strip_render_pipeline {
      pipeline.vertices.clear();
      pipeline.indices.clear();
      pipeline.upload_buffers = true;
    }
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.vertices.clear();
      pipeline.indices.clear();
      pipeline.upload_buffers = true;
    }
  }

  pub fn render<'a>(&mut self, gfx: &Gfx, frame: &mut Frame<'a>, model_view_projection: Mat4) {
    if self.line_list_render_pipeline.is_none() && self.line_strip_render_pipeline.is_none() {
      return; // Nothing to do
    }

    self.uniform_buffer.enqueue_write_all_data(&gfx.queue, &[Uniform { model_view_projection }]);

    if let Some(pipeline) = &mut self.point_list_render_pipeline {
      pipeline.upload_buffers_if_needed(gfx);
    }
    if let Some(pipeline) = &mut self.line_list_render_pipeline {
      pipeline.upload_buffers_if_needed(gfx);
    }
    if let Some(pipeline) = &mut self.line_strip_render_pipeline {
      pipeline.upload_buffers_if_needed(gfx);
    }
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.upload_buffers_if_needed(gfx);
    }

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Debug render pass")
      .begin_render_pass_for_gfx_frame_with_load(gfx, frame, false);
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
    if let Some(pipeline) = &mut self.line_triangle_list_render_pipeline {
      pipeline.draw(&mut render_pass);
    }
  }
}

// Pipeline

struct DebugRendererPipeline<V> {
  render_pipeline: RenderPipeline,
  vertex_buffer: GfxBuffer,
  index_buffer: GfxBuffer,
  upload_buffers: bool,
  vertices: Vec<V>,
  indices: Vec<u32>,
  label: &'static str,
}

impl<V: Vertex + Pod> DebugRendererPipeline<V> {
  fn new(
    gfx: &Gfx,
    vertex_shader_module: &ShaderModule,
    fragment_shader_module: &ShaderModule,
    uniform_bind_group_layout: &BindGroupLayout,
    multisample_sample_count: u32,
    primitive_topology: PrimitiveTopology,
    polygon_mode: PolygonMode,
    label: &'static str,
  ) -> Self {
    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_premultiplied_alpha_blending_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[V::buffer_layout()])
      .with_multisample_count(multisample_sample_count)
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
      .create_with_data(&gfx.device, &vertices);
    let indices = Vec::new();
    let index_buffer = BufferBuilder::new()
      .with_static_index_usage()
      .with_label(&format!("Debug {} index buffer", label))
      .create_with_data(&gfx.device, &indices);
    Self {
      render_pipeline,
      vertex_buffer,
      index_buffer,
      upload_buffers: false,
      vertices,
      indices,
      label,
    }
  }

  #[inline]
  fn push_vertex_without_index(&mut self, vertex: V) {
    self.vertices.push(vertex);
  }

  #[inline]
  fn next_index(&self) -> u32 {
    self.vertices.len() as u32
  }

  #[inline]
  fn push_vertex(&mut self, vertex: V) -> u32 {
    let index = self.next_index();
    self.push_index(index);
    self.push_vertex_without_index(vertex);
    index
  }

  #[inline]
  fn push_index(&mut self, index: u32) {
    self.indices.push(index);
  }

  #[inline]
  fn push_vertices(&mut self, vertices: impl IntoIterator<Item=V>) {
    let base = self.next_index();
    self.vertices.extend(vertices);
    self.indices.extend(base..self.next_index());
  }

  #[inline]
  fn push_vertices_and_indices(&mut self, vertices: impl IntoIterator<Item=V>, indices: impl IntoIterator<Item=u32>) {
    let base = self.vertices.len() as u32;
    self.vertices.extend(vertices);
    self.indices.extend(indices.into_iter().map(|idx| base + idx));
  }

  #[inline]
  fn upload_buffers_if_needed(&mut self, gfx: &Gfx) {
    if self.upload_buffers {
      self.vertex_buffer = BufferBuilder::new()
        .with_static_vertex_usage()
        .with_label(&format!("Debug {} vertex buffer", self.label))
        .create_with_data(&gfx.device, &self.vertices);
      self.index_buffer = BufferBuilder::new()
        .with_static_index_usage()
        .with_label(&format!("Debug {} index buffer", self.label))
        .create_with_data(&gfx.device, &self.indices);
    }
    self.upload_buffers = false;
  }

  #[inline]
  fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
    render_pass.push_debug_group(&format!("Debug draw {}", self.label));
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    if !self.indices.is_empty() {
      render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
      render_pass.draw_indexed(0..self.index_buffer.count() as u32, 0, 0..1);
    } else {
      render_pass.draw(0..self.vertex_buffer.count() as u32, 0..1);
    }
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
pub struct PointVertex {
  pos: Vec3,
  col: Vec4,
  size: f32,
}

impl PointVertex {
  #[inline]
  pub fn new(pos: Vec3, col: Vec4, size: f32) -> Self {
    Self { pos, col, size }
  }
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

// Regular vertex data

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct RegularVertex {
  pos: Vec3,
  col: Vec4,
}

impl RegularVertex {
  #[inline]
  pub fn new(pos: Vec3, col: Vec4) -> Self {
    Self { pos, col }
  }
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
