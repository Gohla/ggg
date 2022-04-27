use std::ops::Range;

use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BufferAddress, Device};

use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::debug_renderer::DebugRenderer;

use crate::chunk::{ChunkVertices, Vertex};
use crate::lod::chunk_vertices::{LodChunkVertices, LodChunkVerticesManager};

// Trait

pub trait LodRenderDataManager {
  fn update(
    &mut self,
    position: Vec3,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> LodRenderData;
}

pub struct LodRenderData {
  pub vertex_buffer: GfxBuffer,
  pub index_buffer: GfxBuffer,
  pub draws: Vec<LodDraw>,
  pub model: Mat4,
}

#[derive(Default, Clone, Debug)]
pub struct LodDraw {
  pub indices: Range<u32>,
  pub base_vertex: u64,
}

// Implementation

pub trait LodRenderDataUpdater {
  type Chunk: LodChunkVertices;

  fn update_chunk(
    &mut self,
    chunk: &Self::Chunk,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    draws: &mut Vec<LodDraw>,
  );
}

#[derive(Default, Debug)]
pub struct SimpleLodRenderDataManager<M, U> {
  pub chunk_vertices_manager: M,
  pub updater: U,
  pub settings: SimpleLodRenderDataSettings,
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
  draws: Vec<LodDraw>,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct SimpleLodRenderDataSettings {
  pub debug_render_octree_nodes: bool,
  pub debug_render_octree_node_color: Vec4,
  pub debug_render_octree_node_empty_color: Vec4,
}

impl<C: LodChunkVertices, M: LodChunkVerticesManager<C>, U> SimpleLodRenderDataManager<M, U> where U: LodRenderDataUpdater<Chunk=C> {
  pub fn new(chunk_vertices_manager: M, updater: U, settings: SimpleLodRenderDataSettings) -> Self {
    let vertices = Vec::new();
    let indices = Vec::new();
    let draws = Vec::new();
    Self { chunk_vertices_manager, updater, settings, vertices, indices, draws }
  }
}

impl<C: LodChunkVertices, T: LodChunkVerticesManager<C>, U> LodRenderDataManager for SimpleLodRenderDataManager<T, U> where U: LodRenderDataUpdater<Chunk=C> {
  fn update(
    &mut self,
    position: Vec3,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> LodRenderData {
    self.vertices.clear();
    self.indices.clear();
    self.draws.clear();

    let (transform, chunks) = self.chunk_vertices_manager.update(position);

    for (aabb, (chunk, filled)) in chunks {
      let is_empty = chunk.is_empty();
      if !is_empty {
        if !*filled { continue; }
        self.updater.update_chunk(chunk, &mut self.vertices, &mut self.indices, &mut self.draws);
      }
      if self.settings.debug_render_octree_nodes {
        if is_empty {
          debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, self.settings.debug_render_octree_node_empty_color);
        } else {
          debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, self.settings.debug_render_octree_node_color);
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

    LodRenderData { vertex_buffer, index_buffer, draws: self.draws.clone(), model: transform.into_homogeneous_matrix() }
  }
}

pub(crate) fn copy_chunk_vertices(
  chunk_vertices: &ChunkVertices,
  vertices: &mut Vec<Vertex>,
  indices: &mut Vec<u16>,
  draws: &mut Vec<LodDraw>,
) {
  if !chunk_vertices.is_empty() {
    let vertex_offset = vertices.len() as BufferAddress;
    let index_offset = indices.len() as u32;
    vertices.extend(chunk_vertices.vertices());
    indices.extend(chunk_vertices.indices());
    draws.push(LodDraw { indices: index_offset..index_offset + chunk_vertices.indices().len() as u32, base_vertex: vertex_offset });
  }
}
