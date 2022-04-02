use wgpu::{BindGroup, CommandEncoder, Device, Face, IndexFormat, Operations, Queue, RenderPass, RenderPipeline, ShaderStages, TextureFormat, TextureView};

use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::surface::GfxSurface;

use crate::chunk::{ChunkVertices, Vertex};
use crate::lod::mesh::LodMesh;
use crate::uniform::{CameraUniform, LightUniform, ModelUniform};

pub struct VoxelRenderer {
  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  model_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  render_pipeline: RenderPipeline,
}

impl VoxelRenderer {
  pub fn new(
    device: &Device,
    surface: &GfxSurface,
    camera_uniform: CameraUniform,
    light_uniform: LightUniform,
    model_uniform: ModelUniform,
    multisample_count: u32,
    cull_mode: Option<Face>,
    depth_texture_format: TextureFormat,
  ) -> Self {
    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .build_with_data(device, &[camera_uniform]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) = camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(device, &[light_uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) = light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let model_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Model uniform buffer")
      .build_with_data(device, &[model_uniform]);
    let (model_uniform_bind_group_layout_entry, model_uniform_bind_group_entry) = model_uniform_buffer.create_uniform_binding_entries(2, ShaderStages::VERTEX);

    let vertex_shader_module = device.create_shader_module(&gfx::include_shader!("vert"));
    let fragment_shader_module = device.create_shader_module(&gfx::include_shader!("frag"));

    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry, model_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry, model_uniform_bind_group_entry])
      .with_layout_label("Voxel renderer uniform bind group layout")
      .with_label("Voxel renderer uniform bind group")
      .build(device);

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .with_multisample_count(multisample_count)
      .with_cull_mode(cull_mode)
      .with_depth_texture(depth_texture_format)
      .with_layout_label("Voxel renderer pipeline layout")
      .with_label("Voxel renderer render pipeline")
      .build(device);

    Self {
      camera_uniform_buffer,
      light_uniform_buffer,
      model_uniform_buffer,
      uniform_bind_group,
      render_pipeline,
    }
  }

  pub fn update_camera_uniform(&mut self, queue: &Queue, camera_uniform: CameraUniform) {
    self.camera_uniform_buffer.write_whole_data(queue, &[camera_uniform]);
  }

  pub fn update_light_uniform(&mut self, queue: &Queue, light_uniform: LightUniform) {
    self.light_uniform_buffer.write_whole_data(queue, &[light_uniform]);
  }

  pub fn update_model_uniform(&mut self, queue: &Queue, model_uniform: ModelUniform) {
    self.model_uniform_buffer.write_whole_data(queue, &[model_uniform]);
  }

  pub fn render_lod_mesh(
    &self,
    depth_texture: &TextureView,
    encoder: &mut CommandEncoder,
    multisampled_framebuffer: Option<&TextureView>,
    framebuffer: &TextureView,
    lod_mesh: &LodMesh,
  ) {
    let mut render_pass = self.create_render_pass(depth_texture, encoder, multisampled_framebuffer, framebuffer);
    render_pass.push_debug_group("Render LOD mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(lod_mesh.index_buffer.slice(..), IndexFormat::Uint16);
    for draw in &lod_mesh.draws {
      render_pass.set_vertex_buffer(0, lod_mesh.vertex_buffer.offset::<Vertex>(draw.base_vertex));
      render_pass.draw_indexed(draw.indices.clone(), 0, 0..1);
    }
    render_pass.pop_debug_group();
  }

  pub fn render_chunk_vertices(
    &self,
    device: &Device,
    depth_texture: &TextureView,
    encoder: &mut CommandEncoder,
    multisampled_framebuffer: Option<&TextureView>,
    framebuffer: &TextureView,
    chunk_vertices: &ChunkVertices,
  ) {
    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Transvoxel demo vertex buffer")
      .build_with_data(device, &chunk_vertices.vertices());
    let index_buffer = BufferBuilder::new()
      .with_index_usage()
      .with_label("Transvoxel demo index buffer")
      .build_with_data(device, &chunk_vertices.indices());
    let mut render_pass = self.create_render_pass(depth_texture, encoder, multisampled_framebuffer, framebuffer);
    render_pass.push_debug_group("Render chunk vertices");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw_indexed(0..index_buffer.len as u32, 0, 0..1);
    render_pass.pop_debug_group();
  }

  fn create_render_pass<'a>(
    &'a self,
    depth_texture: &'a TextureView,
    encoder: &'a mut CommandEncoder,
    multisampled_framebuffer: Option<&'a TextureView>,
    framebuffer: &'a TextureView,
  ) -> RenderPass {
    RenderPassBuilder::new()
      .with_depth_texture(depth_texture)
      .with_label("Voxel render pass")
      .begin_render_pass_for_possibly_multisampled_swap_chain(encoder, multisampled_framebuffer, framebuffer, Operations::default())
  }
}
