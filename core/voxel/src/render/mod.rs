use wgpu::{BindGroup, Face, IndexFormat, Queue, RenderPass, RenderPipeline, ShaderStages};
use wgpu::util::StagingBelt;

use gfx::{Gfx, include_spirv_shader, Render};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::growable_buffer::{GrowableBuffer, GrowableBufferBuilder};
use gfx::render_pass::RenderPassBuilder;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::lod::render::LodRenderData;
use crate::uniform::{CameraUniform, LightUniform, ModelUniform};

pub struct VoxelRenderer {
  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  model_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  render_pipeline: RenderPipeline,
  staging_belt: StagingBelt,
  vertex_buffer: GrowableBuffer,
  index_buffer: GrowableBuffer,
}

impl VoxelRenderer {
  pub fn new(
    gfx: &Gfx,
    camera_uniform: CameraUniform,
    light_uniform: LightUniform,
    model_uniform: ModelUniform,
    cull_mode: Option<Face>,
    staging_belt: StagingBelt,
  ) -> Self {
    let camera_uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Camera uniform buffer")
      .build_with_data(&gfx.device, &[camera_uniform]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Light uniform buffer")
      .build_with_data(&gfx.device, &[light_uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let model_uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Model uniform buffer")
      .build_with_data(&gfx.device, &[model_uniform]);
    let (model_uniform_bind_group_layout_entry, model_uniform_bind_group_entry) = model_uniform_buffer.create_uniform_binding_entries(2, ShaderStages::VERTEX);

    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader!("render/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader!("render/frag"));

    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_label("Voxel renderer uniform bind group layout")
      .with_label("Voxel renderer uniform bind group")
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry, model_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry, model_uniform_bind_group_entry])
      .build(&gfx.device);

    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Voxel renderer pipeline layout")
      .bind_group_layouts(&[&uniform_bind_group_layout])
      .label("Voxel renderer render pipeline")
      .vertex_module(&vertex_shader_module)
      .vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .cull_mode(cull_mode)
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);

    let vertex_buffer = GrowableBufferBuilder::default()
      .vertex_usage()
      .label("Voxel renderer vertex buffer")
      .build();
    let index_buffer = GrowableBufferBuilder::default()
      .index_usage()
      .label("Voxel renderer index buffer")
      .build();

    Self {
      camera_uniform_buffer,
      light_uniform_buffer,
      model_uniform_buffer,
      uniform_bind_group,
      render_pipeline,
      staging_belt,
      vertex_buffer,
      index_buffer,
    }
  }

  pub fn update_camera_uniform(&mut self, queue: &Queue, camera_uniform: CameraUniform) {
    self.camera_uniform_buffer.write_all_data(queue, &[camera_uniform]);
  }

  pub fn update_light_uniform(&mut self, queue: &Queue, light_uniform: LightUniform) {
    self.light_uniform_buffer.write_all_data(queue, &[light_uniform]);
  }

  pub fn update_model_uniform(&mut self, queue: &Queue, model_uniform: ModelUniform) {
    self.model_uniform_buffer.write_all_data(queue, &[model_uniform]);
  }

  #[profiling::function]
  pub fn render_lod_mesh(
    &mut self,
    gfx: &Gfx,
    frame: &mut Render,
    clear: bool,
    lod_mesh: &LodRenderData,
  ) {
    self.staging_belt.recall();
    let vertex_buffer = self.vertex_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &lod_mesh.vertices);
    let index_buffer = self.index_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &lod_mesh.indices);
    let mut render_pass = Self::create_render_pass(gfx, frame, clear);
    render_pass.push_debug_group("Render LOD mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &lod_mesh.draws {
      render_pass.set_vertex_buffer(0, vertex_buffer.slice_data::<Vertex>(draw.base_vertex..));
      render_pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    render_pass.pop_debug_group();
    self.staging_belt.finish();
  }

  #[profiling::function]
  pub fn render_chunk_vertices(
    &mut self,
    gfx: &Gfx,
    frame: &mut Render,
    clear: bool,
    chunk_vertices: &ChunkMesh,
  ) {
    self.staging_belt.recall();
    let vertex_buffer = self.vertex_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &chunk_vertices.vertices());
    let index_buffer = self.index_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &chunk_vertices.indices());
    let mut render_pass = Self::create_render_pass(gfx, frame, clear);
    render_pass.push_debug_group("Render chunk vertices");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw_indexed(0..chunk_vertices.indices().len() as u32, 0, 0..1);
    render_pass.pop_debug_group();
    self.staging_belt.finish();
  }

  #[profiling::function]
  fn create_render_pass<'a>(
    gfx: &'a Gfx,
    frame: &'a mut Render,
    clear: bool,
  ) -> RenderPass<'a> {
    RenderPassBuilder::new()
      .with_label("Voxel render pass")
      .begin_render_pass_for_gfx_frame_simple(gfx, frame, true, clear)
  }
}
