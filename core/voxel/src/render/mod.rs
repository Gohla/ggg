use wgpu::{Face, IndexFormat, Queue, RenderPass, RenderPipeline, ShaderStages};
use wgpu::util::StagingBelt;

use gfx::{Gfx, GfxFrame, include_spirv_shader};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::growable_buffer::{GrowableBuffer, GrowableBufferBuilder};

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::lod::render::LodRenderData;
use crate::uniform::{CameraUniform, LightUniform, ModelUniform};

pub struct VoxelRenderer {
  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  model_uniform_buffer: GfxBuffer,
  uniform_bind_group: CombinedBindGroup,
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
    let camera_uniform_binding = camera_uniform_buffer.binding(0, ShaderStages::VERTEX_FRAGMENT);

    let light_uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Light uniform buffer")
      .build_with_data(&gfx.device, &[light_uniform]);
    let light_uniform_binding = light_uniform_buffer.binding(1, ShaderStages::FRAGMENT);

    let model_uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .label("Model uniform buffer")
      .build_with_data(&gfx.device, &[model_uniform]);
    let model_uniform_binding = model_uniform_buffer.binding(2, ShaderStages::VERTEX);

    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader!("render/vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader!("render/frag"));

    let uniform_bind_group = CombinedBindGroupBuilder::new()
      .layout_label("Voxel renderer uniform bind group layout")
      .label("Voxel renderer uniform bind group")
      .layout_entries(&[camera_uniform_binding.layout, light_uniform_binding.layout, model_uniform_binding.layout])
      .entries(&[camera_uniform_binding.entry, light_uniform_binding.entry, model_uniform_binding.entry])
      .build(&gfx.device);

    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Voxel renderer pipeline layout")
      .bind_group_layouts(&[&uniform_bind_group.layout])
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
    frame: &mut GfxFrame,
    clear: bool,
    lod_mesh: &LodRenderData,
  ) {
    self.staging_belt.recall();
    let vertex_buffer = self.vertex_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &lod_mesh.vertices);
    let index_buffer = self.index_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &lod_mesh.indices);
    let mut pass = Self::create_render_pass(frame, clear);
    pass.push_debug_group("Render LOD mesh");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.uniform_bind_group.entry, &[]);
    pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &lod_mesh.draws {
      pass.set_vertex_buffer(0, vertex_buffer.slice_data::<Vertex>(draw.base_vertex..));
      pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    pass.pop_debug_group();
    self.staging_belt.finish();
  }

  #[profiling::function]
  pub fn render_chunk_vertices(
    &mut self,
    gfx: &Gfx,
    frame: &mut GfxFrame,
    clear: bool,
    chunk_vertices: &ChunkMesh,
  ) {
    self.staging_belt.recall();
    let vertex_buffer = self.vertex_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &chunk_vertices.vertices());
    let index_buffer = self.index_buffer.write_data(&gfx.device, &mut frame.encoder, &mut self.staging_belt, &chunk_vertices.indices());
    let mut pass = Self::create_render_pass(frame, clear);
    pass.push_debug_group("Render chunk vertices");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.uniform_bind_group.entry, &[]);
    pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    pass.draw_indexed(0..chunk_vertices.indices().len() as u32, 0, 0..1);
    pass.pop_debug_group();
    self.staging_belt.finish();
  }

  fn create_render_pass<'a>(frame: &'a mut GfxFrame, clear: bool) -> RenderPass<'a> {
    frame.render_pass_builder()
      .label("Voxel render pass")
      .clear_default_or_load(clear)
      .begin()
  }
}
