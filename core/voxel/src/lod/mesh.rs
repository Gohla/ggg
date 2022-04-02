use std::ops::Range;

use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BufferAddress, Device};

use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::debug_renderer::DebugRenderer;

use crate::chunk::{ChunkVertices, Vertex};
use crate::lod::chunk::LodChunkManager;

#[derive(Default, Debug)]
pub struct LodMeshManager {
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
  draws: Vec<LodDraw>,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct LodMeshManagerSettings {
  pub render_regular_chunks: bool,
  pub render_transition_lo_x_chunks: bool,
  pub render_transition_hi_x_chunks: bool,
  pub render_transition_lo_y_chunks: bool,
  pub render_transition_hi_y_chunks: bool,
  pub render_transition_lo_z_chunks: bool,
  pub render_transition_hi_z_chunks: bool,
  pub debug_render_octree_nodes: bool,
  pub debug_render_octree_node_color: Vec4,
  pub debug_render_octree_node_empty_color: Vec4,
}

#[derive(Default, Clone, Debug)]
pub struct LodDraw {
  pub indices: Range<u32>,
  pub base_vertex: u64,
}

pub struct LodMesh {
  pub vertex_buffer: GfxBuffer,
  pub index_buffer: GfxBuffer,
  pub draws: Vec<LodDraw>,
  pub model: Mat4,
}

impl LodMeshManager {
  pub fn new() -> Self {
    let vertices = Vec::new();
    let indices = Vec::new();
    let draws = Vec::new();
    Self { vertices, indices, draws }
  }

  pub fn update(
    &mut self,
    lod_chunk_manager: &mut dyn LodChunkManager,
    position: Vec3,
    settings: &LodMeshManagerSettings,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> LodMesh {
    self.vertices.clear();
    self.indices.clear();
    self.draws.clear();

    let (transform, chunks) = lod_chunk_manager.update(position);

    for (aabb, (chunk, filled)) in chunks {
      let is_empty = chunk.regular.is_empty();
      if !*filled { continue; }
      if settings.render_regular_chunks {
        self.copy_chunk_vertices(&chunk.regular);
      }
      if settings.render_transition_lo_x_chunks {
        self.copy_chunk_vertices(&chunk.transition_lo_x_chunk);
      }
      if settings.render_transition_hi_x_chunks {
        self.copy_chunk_vertices(&chunk.transition_hi_x_chunk);
      }
      if settings.render_transition_lo_y_chunks {
        self.copy_chunk_vertices(&chunk.transition_lo_y_chunk);
      }
      if settings.render_transition_hi_y_chunks {
        self.copy_chunk_vertices(&chunk.transition_hi_y_chunk);
      }
      if settings.render_transition_lo_z_chunks {
        self.copy_chunk_vertices(&chunk.transition_lo_z_chunk);
      }
      if settings.render_transition_hi_z_chunks {
        self.copy_chunk_vertices(&chunk.transition_hi_z_chunk);
      }
      if settings.debug_render_octree_nodes {
        if is_empty {
          debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_empty_color);
        } else {
          debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_color);
        }
      }
    }

    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(device, &self.vertices);
    let index_buffer = BufferBuilder::new()
      .with_index_usage()
      .with_label("Voxel meshing index buffer")
      .build_with_data(device, &self.indices);

    LodMesh { vertex_buffer, index_buffer, draws: self.draws.clone(), model: transform.into_homogeneous_matrix() }
  }

  fn copy_chunk_vertices(
    &mut self,
    chunk_vertices: &ChunkVertices,
  ) {
    if !chunk_vertices.is_empty() {
      let vertex_offset = self.vertices.len() as BufferAddress;
      let index_offset = self.indices.len() as u32;
      self.vertices.extend(chunk_vertices.vertices());
      self.indices.extend(chunk_vertices.indices());
      self.draws.push(LodDraw { indices: index_offset..index_offset + chunk_vertices.indices().len() as u32, base_vertex: vertex_offset });
    }
  }
}
