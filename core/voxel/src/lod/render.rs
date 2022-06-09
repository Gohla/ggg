use std::borrow::Borrow;
use std::ops::Range;

use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BufferAddress, Device};

use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::debug_renderer::DebugRenderer;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::size::ChunkSize;
use crate::lod::chunk_mesh::{LodChunkMesh, LodChunkMeshManager, LodChunkMeshManagerParameters};
use crate::lod::extract::LodExtractor;
use crate::lod::octmap::LodJobOutput;

// Trait

pub trait LodRenderDataManager<C: ChunkSize> {
  fn update(
    &mut self,
    position: Vec3,
    settings: &LodRenderDataSettings,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> LodRenderData;

  fn get_mesh_manager_parameters_mut(&mut self) -> &mut dyn LodChunkMeshManagerParameters;
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LodRenderDataSettings {
  pub debug_render_vertices: bool,
  pub debug_render_vertex_color: Vec4,
  pub debug_render_vertex_point_size: f32,
  pub debug_render_edges: bool,
  pub debug_render_edge_color: Vec4,
  pub debug_render_octree_nodes: bool,
  pub debug_render_octree_node_color: Vec4,
  pub debug_render_octree_node_empty_color: Vec4,
}

impl Default for LodRenderDataSettings {
  fn default() -> Self {
    Self {
      debug_render_vertices: true,
      debug_render_vertex_color: Vec4::new(0.0, 0.0, 0.5, 0.5),
      debug_render_vertex_point_size: 3.0,
      debug_render_edges: false,
      debug_render_edge_color: Vec4::new(0.0, 0.25, 0.0, 0.25),
      debug_render_octree_nodes: true,
      debug_render_octree_node_color: Vec4::new(0.0, 0.1, 0.0, 0.1),
      debug_render_octree_node_empty_color: Vec4::new(0.1, 0.0, 0.0, 0.1),
    }
  }
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

#[derive(Default, Debug)]
pub struct SimpleLodRenderDataManager<MM> {
  chunk_mesh_manager: MM,
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
  draws: Vec<LodDraw>,
}

impl<MM> SimpleLodRenderDataManager<MM> {
  pub fn new(chunk_mesh_manager: MM) -> Self {
    let vertices = Vec::new();
    let indices = Vec::new();
    let draws = Vec::new();
    Self { chunk_mesh_manager, vertices, indices, draws }
  }
}

impl<C: ChunkSize, E: LodExtractor<C>, MM> LodRenderDataManager<C> for SimpleLodRenderDataManager<MM> where
  MM: LodChunkMeshManager<C, Extractor=E>
{
  #[profiling::function]
  fn update(
    &mut self,
    position: Vec3,
    settings: &LodRenderDataSettings,
    debug_renderer: &mut DebugRenderer,
    device: &Device,
  ) -> LodRenderData {
    self.vertices.clear();
    self.indices.clear();
    self.draws.clear();

    let extractor = self.chunk_mesh_manager.get_extractor().clone();
    let (transform, outputs) = self.chunk_mesh_manager.update(position);

    for (aabb, output) in outputs {
      if let LodJobOutput::Mesh(lod_chunk_mesh) = output.borrow() {
        let is_empty = lod_chunk_mesh.is_empty();
        if !is_empty {
          extractor.update_render_data(lod_chunk_mesh, &mut self.vertices, &mut self.indices, &mut self.draws);
        }
        if settings.debug_render_octree_nodes {
          if is_empty {
            debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_empty_color);
          } else {
            debug_renderer.draw_cube_lines(aabb.min().into(), aabb.size() as f32, settings.debug_render_octree_node_color);
          }
        }
      }
    }

    if settings.debug_render_vertices {
      debug_renderer.draw_points(self.vertices.iter().map(|v| v.position), settings.debug_render_vertex_color, settings.debug_render_vertex_point_size)
    }
    if settings.debug_render_edges {
      for draw in &self.draws {
        debug_renderer.draw_triangles_wireframe_indexed(
          self.vertices.iter().map(|v| v.position),
          self.indices[draw.indices.start as usize..draw.indices.end as usize].iter().map(|i| draw.base_vertex as u32 + *i as u32),
          settings.debug_render_edge_color,
        );
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

  #[inline]
  fn get_mesh_manager_parameters_mut(&mut self) -> &mut dyn LodChunkMeshManagerParameters {
    &mut self.chunk_mesh_manager
  }
}

#[profiling::function]
pub(crate) fn copy_chunk_vertices(
  chunk_vertices: &ChunkMesh,
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
