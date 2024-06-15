use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BufferAddress, Features, PolygonMode, PrimitiveTopology, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};
use wgpu::util::StagingBelt;

use crate::{Gfx, include_spirv_shader, Render};
use crate::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use crate::buffer::{BufferBuilder, GfxBuffer};
use crate::debug_renderer::pipeline::{Pipeline, Vertex};

mod pipeline;

pub struct DebugRenderer {
  staging_belt: StagingBelt,
  uniform_buffer: GfxBuffer,
  uniform_bind_group: CombinedBindGroup,

  point_list_pipeline: Option<Pipeline<PointVertex>>,
  line_list_pipeline: Option<Pipeline<RegularVertex>>,
  line_strip_pipeline: Option<Pipeline<RegularVertex>>,
  line_triangle_list_pipeline: Option<Pipeline<RegularVertex>>,
}

impl DebugRenderer {
  pub fn request_features() -> Features {
    Features::POLYGON_MODE_POINT | Features::POLYGON_MODE_LINE
  }

  pub fn new(gfx: &Gfx, view_projection: Mat4) -> Self {
    let staging_belt = StagingBelt::new(1024 * 1024); // 1 MiB chunk size

    let uniform_buffer = BufferBuilder::new()
      .uniform_usage()
      .label("Debug uniform buffer")
      .build_with_data(&gfx.device, &[Uniform { model_view_projection: view_projection }]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::VERTEX_FRAGMENT);
    let uniform_bind_group = CombinedBindGroupBuilder::default()
      .layout_label("Debug uniform bind group layout")
      .layout_entries(&[uniform_binding.layout])
      .label("Debug uniform bind group")
      .entries(&[uniform_binding.entry])
      .build(&gfx.device);

    let point_vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader!("debug_renderer/point_vert"));
    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader!("debug_renderer/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader!("debug_renderer/frag"));

    let has_polygon_mode_point_feature = gfx.adapter.features().contains(Features::POLYGON_MODE_POINT);
    let point_list_pipeline = has_polygon_mode_point_feature.then(|| Pipeline::new(
      gfx,
      &point_vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group.layout,
      PrimitiveTopology::PointList,
      PolygonMode::Point,
      "point list",
    ));

    let has_polygon_mode_line_feature = gfx.adapter.features().contains(Features::POLYGON_MODE_LINE);
    let line_list_pipeline = has_polygon_mode_line_feature.then(|| Pipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group.layout,
      PrimitiveTopology::LineList,
      PolygonMode::Line,
      "line list",
    ));
    let line_strip_pipeline = has_polygon_mode_line_feature.then(|| Pipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group.layout,
      PrimitiveTopology::LineStrip,
      PolygonMode::Line,
      "line strip",
    ));
    let line_triangle_list_pipeline = has_polygon_mode_line_feature.then(|| Pipeline::new(
      gfx,
      &vertex_shader_module,
      &fragment_shader_module,
      &uniform_bind_group.layout,
      PrimitiveTopology::TriangleList,
      PolygonMode::Line,
      "line triangle list",
    ));

    Self {
      staging_belt,
      uniform_buffer,
      uniform_bind_group,
      point_list_pipeline,
      line_list_pipeline,
      line_strip_pipeline,
      line_triangle_list_pipeline,
    }
  }


  pub fn draw_point(&mut self, point: Vec3, col: Vec4, size: f32) {
    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.push_vertex(PointVertex::new(point, col, size));
    }
  }

  pub fn draw_points(&mut self, positions: impl IntoIterator<Item=Vec3>, col: Vec4, size: f32) {
    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| PointVertex::new(pos, col, size)));
    }
  }

  pub fn draw_point_vertices(&mut self, vertices: impl IntoIterator<Item=PointVertex>) {
    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.push_vertices(vertices);
    }
  }


  pub fn draw_line(&mut self, start_pos: Vec3, end_pos: Vec3, start_col: Vec4, end_col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.push_vertex(RegularVertex::new(start_pos, start_col));
      pipeline.push_vertex(RegularVertex::new(end_pos, end_col));
    }
  }

  pub fn draw_lines(&mut self, positions: impl IntoIterator<Item=Vec3>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)));
    }
  }

  pub fn draw_lines_indexed(&mut self, positions: impl IntoIterator<Item=Vec3>, indices: impl IntoIterator<Item=u32>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.push_vertices_and_indices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)), indices);
    }
  }

  pub fn draw_line_vertices(&mut self, vertices: impl IntoIterator<Item=RegularVertex>) {
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.push_vertices(vertices);
    }
  }

  pub fn draw_line_vertices_indexed(&mut self, vertices: impl IntoIterator<Item=RegularVertex>, indices: impl IntoIterator<Item=u32>) {
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.push_vertices_and_indices(vertices, indices);
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
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.push_vertices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)));
    }
  }

  pub fn draw_triangles_wireframe_indexed(&mut self, positions: impl IntoIterator<Item=Vec3>, indices: impl IntoIterator<Item=u32>, col: Vec4) {
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.push_vertices_and_indices(positions.into_iter().map(|pos| RegularVertex::new(pos, col)), indices);
    }
  }

  pub fn draw_triangle_vertices_wireframe(&mut self, vertices: impl IntoIterator<Item=RegularVertex>) {
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.push_vertices(vertices);
    }
  }

  pub fn draw_triangle_vertices_wireframe_indexed(&mut self, vertices: impl IntoIterator<Item=RegularVertex>, indices: impl IntoIterator<Item=u32>) {
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.push_vertices_and_indices(vertices, indices);
    }
  }


  pub fn clear(&mut self) {
    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.clear();
    }
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.clear();
    }
    if let Some(pipeline) = &mut self.line_strip_pipeline {
      pipeline.clear();
    }
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.clear();
    }
  }

  pub fn render(&mut self, gfx: &Gfx, render: &mut Render, model_view_projection: Mat4) {
    if self.point_list_pipeline.is_some() && self.line_list_pipeline.is_none() &&
      self.line_strip_pipeline.is_none() && self.line_triangle_list_pipeline.is_none() {
      return; // Nothing to do
    }

    self.staging_belt.recall();

    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform { model_view_projection }]);

    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.write_buffers_if_needed(gfx, &mut render.encoder, &mut self.staging_belt);
    }
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.write_buffers_if_needed(gfx, &mut render.encoder, &mut self.staging_belt);
    }
    if let Some(pipeline) = &mut self.line_strip_pipeline {
      pipeline.write_buffers_if_needed(gfx, &mut render.encoder, &mut self.staging_belt);
    }
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.write_buffers_if_needed(gfx, &mut render.encoder, &mut self.staging_belt);
    }

    let mut render_pass = render.render_pass_builder_without_depth_stencil()
      .label("Debug render pass")
      .load()
      .begin();
    render_pass.set_bind_group(0, &self.uniform_bind_group.entry, &[]);
    if let Some(pipeline) = &mut self.point_list_pipeline {
      pipeline.draw(&mut render_pass);
    }
    if let Some(pipeline) = &mut self.line_list_pipeline {
      pipeline.draw(&mut render_pass);
    }
    if let Some(pipeline) = &mut self.line_strip_pipeline {
      pipeline.draw(&mut render_pass);
    }
    if let Some(pipeline) = &mut self.line_triangle_list_pipeline {
      pipeline.draw(&mut render_pass);
    }

    self.staging_belt.finish();

    drop(render_pass);
  }
}

// Uniform data

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  model_view_projection: Mat4,
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
  pub fn new(pos: Vec3, col: Vec4, size: f32) -> Self { Self { pos, col, size } }
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
  pub fn new(pos: Vec3, col: Vec4) -> Self { Self { pos, col } }
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
