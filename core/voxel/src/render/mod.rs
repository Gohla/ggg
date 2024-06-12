use wgpu::{BindGroup, Face, IndexFormat, Queue, RenderPass, RenderPipeline, ShaderStages};
use wgpu::util::StagingBelt;

use gfx::{Frame, Gfx};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::growable_buffer::{GrowableBuffer, GrowableBufferBuilder};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::lod::render::LodRenderData;
use crate::uniform::{CameraUniform, LightUniform, ModelUniform};

pub struct VoxelRenderer {
  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  model_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  render_pipeline: RenderPipeline,
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
  ) -> Self {
    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .create_with_data(&gfx.device, &[camera_uniform]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .create_with_data(&gfx.device, &[light_uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let model_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Model uniform buffer")
      .create_with_data(&gfx.device, &[model_uniform]);
    let (model_uniform_bind_group_layout_entry, model_uniform_bind_group_entry) = model_uniform_buffer.create_uniform_binding_entries(2, ShaderStages::VERTEX);

    let vertex_shader_module = gfx.device.create_shader_module(gfx::include_shader!("render/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(gfx::include_shader!("render/frag"));

    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry, model_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry, model_uniform_bind_group_entry])
      .with_layout_label("Voxel renderer uniform bind group layout")
      .with_label("Voxel renderer uniform bind group")
      .build(&gfx.device);

    let mut render_pipeline_builder = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_cull_mode(cull_mode)
      .with_layout_label("Voxel renderer pipeline layout")
      .with_label("Voxel renderer render pipeline");
    if let Some(depth_texture) = &gfx.depth_stencil_texture {
      render_pipeline_builder = render_pipeline_builder.with_depth_texture(depth_texture.format);
    }
    if gfx.sample_count > 1 {
      render_pipeline_builder = render_pipeline_builder.with_multisample_count(gfx.sample_count)
    }

    let (_, render_pipeline) = render_pipeline_builder
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .build(&gfx.device);

    let vertex_buffer = GrowableBufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel renderer vertex buffer")
      .create();
    let index_buffer = GrowableBufferBuilder::new()
      .with_index_usage()
      .with_label("Voxel renderer index buffer")
      .create();

    Self {
      camera_uniform_buffer,
      light_uniform_buffer,
      model_uniform_buffer,
      uniform_bind_group,
      render_pipeline,
      vertex_buffer,
      index_buffer,
    }
  }

  pub fn update_camera_uniform(&mut self, queue: &Queue, camera_uniform: CameraUniform) {
    self.camera_uniform_buffer.enqueue_write_all_data(queue, &[camera_uniform]);
  }

  pub fn update_light_uniform(&mut self, queue: &Queue, light_uniform: LightUniform) {
    self.light_uniform_buffer.enqueue_write_all_data(queue, &[light_uniform]);
  }

  pub fn update_model_uniform(&mut self, queue: &Queue, model_uniform: ModelUniform) {
    self.model_uniform_buffer.enqueue_write_all_data(queue, &[model_uniform]);
  }

  #[profiling::function]
  pub fn render_lod_mesh(
    &self,
    gfx: &Gfx,
    frame: &mut Frame,
    clear: bool,
    lod_mesh: &LodRenderData,
  ) {
    let mut render_pass = Self::create_render_pass(gfx, frame, clear);
    render_pass.push_debug_group("Render LOD mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(lod_mesh.index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &lod_mesh.draws {
      render_pass.set_vertex_buffer(0, lod_mesh.vertex_buffer.slice_data::<Vertex>(draw.base_vertex..));
      render_pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    render_pass.pop_debug_group();
  }

  #[profiling::function]
  pub fn render_chunk_vertices(
    &mut self,
    gfx: &Gfx,
    frame: &mut Frame,
    staging_belt: &mut StagingBelt,
    clear: bool,
    chunk_vertices: &ChunkMesh,
  ) {
    let vertex_buffer = self.vertex_buffer.write_data(&gfx.device, &mut frame.encoder, staging_belt, &chunk_vertices.vertices());
    let index_buffer = self.index_buffer.write_data(&gfx.device, &mut frame.encoder, staging_belt, &chunk_vertices.indices());
    let mut render_pass = Self::create_render_pass(gfx, frame, clear);
    render_pass.push_debug_group("Render chunk vertices");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw_indexed(0..chunk_vertices.indices().len() as u32, 0, 0..1);
    render_pass.pop_debug_group();
  }

  #[profiling::function]
  fn create_render_pass<'a>(
    gfx: &'a Gfx,
    frame: &'a mut Frame,
    clear: bool,
  ) -> RenderPass<'a> {
    RenderPassBuilder::new()
      .with_label("Voxel render pass")
      .begin_render_pass_for_gfx_frame_simple(gfx, frame, true, clear)
  }
}
