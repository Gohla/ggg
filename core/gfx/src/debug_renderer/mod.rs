use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, BufferAddress, PolygonMode, PrimitiveTopology, RenderPipeline, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};

use crate::{Frame, Gfx, include_shader};
use crate::bind_group::CombinedBindGroupLayoutBuilder;
use crate::buffer::{BufferBuilder, GfxBuffer};
use crate::render_pass::RenderPassBuilder;
use crate::render_pipeline::RenderPipelineBuilder;

pub struct DebugRenderer {
  uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,

  line_list_render_pipeline: RenderPipeline,
  line_list_vertex_buffer: GfxBuffer,
  upload_new_line_list_vertex_buffer: bool,
  line_list_vertices: Vec<Vertex>,

  line_strip_render_pipeline: RenderPipeline,
  line_strip_vertex_buffer: GfxBuffer,
  upload_new_line_strip_vertex_buffer: bool,
  line_strip_vertices: Vec<Vertex>,
}

impl DebugRenderer {
  pub fn new(gfx: &Gfx, view_projection: Mat4) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("debug_renderer/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("debug_renderer/frag"));

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Debug uniform buffer")
      .build_with_data(&gfx.device, &[Uniform { view_projection }]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Debug uniform bind group layout")
      .with_label("Debug uniform bind group")
      .build(&gfx.device);

    let (_, line_list_render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_alpha_blending_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_primitive_topology(PrimitiveTopology::LineList)
      .with_polygon_mode(PolygonMode::Line)
      .with_cull_mode(None)
      .with_layout_label("Debug line list pipeline layout")
      .with_label("Debug line list render pipeline")
      .build(&gfx.device);
    let line_list_vertices = Vec::new();
    let line_list_vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Debug line list vertex buffer")
      .build_with_data(&gfx.device, &line_list_vertices);

    let (_, line_strip_render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_alpha_blending_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_primitive_topology(PrimitiveTopology::LineStrip)
      .with_polygon_mode(PolygonMode::Line)
      .with_cull_mode(None)
      .with_layout_label("Debug line strip pipeline layout")
      .with_label("Debug line strip render pipeline")
      .build(&gfx.device);
    let line_strip_vertices = Vec::new();
    let line_strip_vertex_buffer = BufferBuilder::new()
      .with_static_vertex_usage()
      .with_label("Debug line strip vertex buffer")
      .build_with_data(&gfx.device, &line_strip_vertices);

    Self {
      uniform_buffer,
      uniform_bind_group,

      line_list_render_pipeline,
      line_list_vertex_buffer,
      upload_new_line_list_vertex_buffer: false,
      line_list_vertices,

      line_strip_render_pipeline,
      line_strip_vertex_buffer,
      upload_new_line_strip_vertex_buffer: false,
      line_strip_vertices,
    }
  }

  pub fn draw_line(&mut self, start_pos: Vec3, end_pos: Vec3, start_col: Vec4, end_col: Vec4) {
    self.line_list_vertices.push(Vertex::new(start_pos, start_col));
    self.line_list_vertices.push(Vertex::new(end_pos, end_col));
    self.upload_new_line_list_vertex_buffer = true;
  }

  pub fn draw_cube(&mut self, min: Vec3, size: f32, col: Vec4) {
    self.line_list_vertices.push(Vertex::new(min, col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, 0.0, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min, col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min, col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, 0.0, size), col));

    let max = min + Vec3::new(size, size, size);
    self.line_list_vertices.push(Vertex::new(max, col));
    self.line_list_vertices.push(Vertex::new(max + Vec3::new(-size, 0.0, 0.0), col));
    self.line_list_vertices.push(Vertex::new(max, col));
    self.line_list_vertices.push(Vertex::new(max + Vec3::new(0.0, -size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(max, col));
    self.line_list_vertices.push(Vertex::new(max + Vec3::new(0.0, 0.0, -size), col));

    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, size, size), col));

    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, 0.0, size), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, 0.0, size), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, 0.0, size), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(0.0, size, size), col));

    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, 0.0, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, size, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, 0.0, 0.0), col));
    self.line_list_vertices.push(Vertex::new(min + Vec3::new(size, 0.0, size), col));

    self.upload_new_line_list_vertex_buffer = true;
  }

  pub fn clear(&mut self) {
    self.line_list_vertices.clear();
    self.upload_new_line_list_vertex_buffer = true;
    self.line_strip_vertices.clear();
    self.upload_new_line_strip_vertex_buffer = true;
  }

  pub fn render<'a>(&mut self, gfx: &Gfx, frame: &mut Frame<'a>, view_projection: Mat4) {
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform { view_projection }]);
    if self.upload_new_line_list_vertex_buffer {
      self.line_list_vertex_buffer = BufferBuilder::new()
        .with_static_vertex_usage()
        .with_label("Debug vertex buffer")
        .build_with_data(&gfx.device, &self.line_list_vertices);
    }
    if self.upload_new_line_strip_vertex_buffer {
      self.line_strip_vertex_buffer = BufferBuilder::new()
        .with_static_vertex_usage()
        .with_label("Debug vertex buffer")
        .build_with_data(&gfx.device, &self.line_strip_vertices);
    }

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Debug render pass")
      .begin_render_pass_for_swap_chain_with_load(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw debug line list");
    render_pass.set_pipeline(&self.line_list_render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.line_list_vertex_buffer.slice(..));
    render_pass.draw(0..self.line_list_vertex_buffer.len as u32, 0..1);
    render_pass.pop_debug_group();
    render_pass.push_debug_group("Draw debug line strip");
    render_pass.set_pipeline(&self.line_strip_render_pipeline);
    render_pass.set_vertex_buffer(0, self.line_strip_vertex_buffer.slice(..));
    render_pass.draw(0..self.line_strip_vertex_buffer.len as u32, 0..1);
    render_pass.pop_debug_group();
  }
}

// Uniform data

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}

// Vertex data

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec3,
  col: Vec4,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
      0 => Float32x3,
      1 => Float32x4,
    ];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }

  #[inline]
  fn new(pos: Vec3, col: Vec4) -> Self {
    Self { pos, col }
  }
}
