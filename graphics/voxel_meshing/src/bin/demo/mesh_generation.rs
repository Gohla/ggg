use std::ops::Range;

use ultraviolet::Vec3;
use wgpu::{BufferAddress, Device};

use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::debug_renderer::DebugRenderer;
use voxel_meshing::chunk::{ChunkVertices, Vertex};
use voxel_meshing::octree::VolumeMeshManager;

use crate::settings::Settings;

pub struct MeshGeneration {
  pub volume_mesh_manager: Box<dyn VolumeMeshManager>,
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
  pub draws: Vec<Draw>,
  pub vertex_buffer: GfxBuffer,
  pub index_buffer: GfxBuffer,
}

pub struct Draw {
  pub indices: Range<u32>,
  pub base_vertex: u64,
}

impl MeshGeneration {
  pub fn new(
    position: Vec3,
    settings: &Settings,
    mut volume_mesh_manager: Box<dyn VolumeMeshManager>,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> Self {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut draws = Vec::new();
    let (vertex_buffer, index_buffer) = Self::do_update(position, settings, &mut vertices, &mut indices, &mut draws, debug_renderer, &mut *volume_mesh_manager, device);
    Self { volume_mesh_manager, vertices, indices, draws, vertex_buffer, index_buffer }
  }

  pub fn update(
    &mut self,
    position: Vec3,
    settings: &Settings,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) {
    let (vertex_buffer, index_buffer) = Self::do_update(position, settings, &mut self.vertices, &mut self.indices, &mut self.draws, debug_renderer, &mut *self.volume_mesh_manager, device);
    self.vertex_buffer = vertex_buffer;
    self.index_buffer = index_buffer;
  }

  pub fn set_volume_mesh_manager(
    &mut self,
    volume_mesh_manager: Box<dyn VolumeMeshManager>,
    position: Vec3,
    settings: &Settings,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) {
    self.volume_mesh_manager = volume_mesh_manager;
    self.update(position, settings, debug_renderer, device);
  }

  fn do_update(
    position: Vec3,
    settings: &Settings,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<Draw>,
    debug_renderer: &mut DebugRenderer,
    volume_mesh_manager: &mut dyn VolumeMeshManager,
    device: &Device,
  ) -> (GfxBuffer, GfxBuffer) {
    vertices.clear();
    indices.clear();
    draws.clear();
    debug_renderer.clear();

    for (aabb, (chunk, filled)) in volume_mesh_manager.update(position) {
      let is_empty = chunk.regular.is_empty();
      if *filled {
        if settings.render_regular_chunks {
          Self::render_chunk(&chunk.regular, vertices, indices, draws);
        }
        if settings.render_transition_lo_x_chunks {
          Self::render_chunk(&chunk.transition_lo_x_chunk, vertices, indices, draws);
        }
        if settings.render_transition_hi_x_chunks {
          Self::render_chunk(&chunk.transition_hi_x_chunk, vertices, indices, draws);
        }
        if settings.render_transition_lo_y_chunks {
          Self::render_chunk(&chunk.transition_lo_y_chunk, vertices, indices, draws);
        }
        if settings.render_transition_hi_y_chunks {
          Self::render_chunk(&chunk.transition_hi_y_chunk, vertices, indices, draws);
        }
        if settings.render_transition_lo_z_chunks {
          Self::render_chunk(&chunk.transition_lo_z_chunk, vertices, indices, draws);
        }
        if settings.render_transition_hi_z_chunks {
          Self::render_chunk(&chunk.transition_hi_z_chunk, vertices, indices, draws);
        }
        if settings.debug_render_octree_nodes {
          if is_empty {
            debug_renderer.draw_cube(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_empty_color);
          } else {
            debug_renderer.draw_cube(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_color);
          }
        }
      }
    }

    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(device, &vertices);
    let index_buffer = BufferBuilder::new()
      .with_index_usage()
      .with_label("Voxel meshing index buffer")
      .build_with_data(device, &indices);
    (vertex_buffer, index_buffer)
  }

  fn render_chunk(
    chunk_vertices: &ChunkVertices,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<Draw>,
  ) {
    if !chunk_vertices.is_empty() {
      let vertex_offset = vertices.len() as BufferAddress;
      let index_offset = indices.len() as u32;
      vertices.extend(chunk_vertices.vertices());
      indices.extend(chunk_vertices.indices());
      draws.push(Draw { indices: index_offset..index_offset + chunk_vertices.indices().len() as u32, base_vertex: vertex_offset });
    }
  }
}
