use std::ops::Range;

use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::Device;

use gfx::{Frame, Gfx};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::debug_renderer::DebugRenderer;

use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::size::ChunkSize;
use crate::lod::chunk_mesh::{LodChunkMesh, LodChunkMeshManager, LodChunkMeshManagerParameters};
use crate::lod::extract::LodExtractor;

// Trait

pub trait LodRenderDataManager<C: ChunkSize> {
  fn update(
    &mut self,
    position: Vec3,
    settings: &LodRenderDataSettings,
    device: &Device,
  ) -> LodRenderData;

  fn debug_render<'a>(&mut self, gfx: &Gfx, frame: &mut Frame<'a>, view_projection_matrix: Mat4, data: &LodRenderData);

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
  pub debug_render_octree_aabb_closest_points: bool,
  pub debug_render_octree_aabb_closest_points_color: Vec4,
  pub debug_render_octree_aabb_closest_points_point_size: f32,
}

impl Default for LodRenderDataSettings {
  fn default() -> Self {
    Self {
      debug_render_vertices: false,
      debug_render_vertex_color: Vec4::new(0.0, 0.0, 0.5, 0.5),
      debug_render_vertex_point_size: 3.0,
      debug_render_edges: false,
      debug_render_edge_color: Vec4::new(0.0, 0.25, 0.0, 0.25),
      debug_render_octree_nodes: true,
      debug_render_octree_node_color: Vec4::new(0.0, 0.1, 0.0, 0.1),
      debug_render_octree_node_empty_color: Vec4::new(0.1, 0.0, 0.0, 0.1),
      debug_render_octree_aabb_closest_points: false,
      debug_render_octree_aabb_closest_points_color: Vec4::new(0.0, 0.0, 0.1, 0.1),
      debug_render_octree_aabb_closest_points_point_size: 3.0,
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
  pub base_vertex: usize,
}

// Implementation

pub struct SimpleLodRenderDataManager<MM> {
  chunk_mesh_manager: MM,
  debug_renderer: DebugRenderer,
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
  draws: Vec<LodDraw>,
}

impl<MM> SimpleLodRenderDataManager<MM> {
  pub fn new(gfx: &Gfx, chunk_mesh_manager: MM, view_projection_matrix: Mat4) -> Self {
    let debug_renderer = DebugRenderer::new(gfx, view_projection_matrix);
    let vertices = Vec::new();
    let indices = Vec::new();
    let draws = Vec::new();
    Self { chunk_mesh_manager, debug_renderer, vertices, indices, draws }
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
    device: &Device,
  ) -> LodRenderData {
    self.debug_renderer.clear();
    self.vertices.clear();
    self.indices.clear();
    self.draws.clear();

    let extractor = self.chunk_mesh_manager.get_extractor().clone();
    let (root_half_size, transform, lod_chunk_meshes) = self.chunk_mesh_manager.update(position);

    // Transform the position into one local to AABBs, used when `debug_render_octree_aabb_closest_points` is true.
    let transform_inverse = transform.inversed();
    let aabb_local_position = transform_inverse.transform_vec(position);

    for (aabb, lod_chunk_mesh) in lod_chunk_meshes {
      let is_empty = lod_chunk_mesh.is_empty();
      if !is_empty {
        extractor.update_render_data(&lod_chunk_mesh, &mut self.vertices, &mut self.indices, &mut self.draws);
      }
      if settings.debug_render_octree_nodes {
        let min = aabb.minimum_point(root_half_size).into();
        let size = aabb.size(root_half_size) as f32;
        if is_empty {
          self.debug_renderer.draw_cube_lines(min, size, settings.debug_render_octree_node_empty_color);
        } else {
          self.debug_renderer.draw_cube_lines(min, size, settings.debug_render_octree_node_color);
        }
      }
      if settings.debug_render_octree_aabb_closest_points {
        // Use `aabb_local_position` here because AABBs are in their own local space. Afterwards, we do not have to
        // transform back because the debug renderer will transform everything into world space using the
        // (non-inverse) transform.
        let aabb_local_closest_point = aabb.closest_point(root_half_size, aabb_local_position);
        let color = settings.debug_render_octree_aabb_closest_points_color;
        self.debug_renderer.draw_point(aabb_local_closest_point, color, settings.debug_render_octree_aabb_closest_points_point_size);
        self.debug_renderer.draw_line(aabb_local_position, aabb_local_closest_point, color, color);
      }
    }

    if settings.debug_render_vertices {
      self.debug_renderer.draw_points(self.vertices.iter().map(|v| v.position), settings.debug_render_vertex_color, settings.debug_render_vertex_point_size)
    }
    if settings.debug_render_edges {
      for draw in &self.draws {
        self.debug_renderer.draw_triangles_wireframe_indexed(
          self.vertices.iter().map(|v| v.position),
          self.indices[draw.indices.start as usize..draw.indices.end as usize].iter().map(|i| draw.base_vertex as u32 + *i as u32),
          settings.debug_render_edge_color,
        );
      }
    }

    // OPTO: don't create new buffers each time, reuse them instead!
    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .create_with_data(device, &self.vertices);
    let index_buffer = BufferBuilder::new()
      .with_index_usage()
      .with_label("Voxel meshing index buffer")
      .create_with_data(device, &self.indices);
    let draws = std::mem::take(&mut self.draws);
    let model = transform.into_homogeneous_matrix();

    LodRenderData { vertex_buffer, index_buffer, draws, model }
  }

  #[inline]
  fn debug_render<'a>(&mut self, gfx: &Gfx, frame: &mut Frame<'a>, view_projection_matrix: Mat4, data: &LodRenderData) {
    self.debug_renderer.render(gfx, frame, view_projection_matrix * data.model);
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
    let vertex_offset = vertices.len();
    let index_offset = indices.len() as u32;
    vertices.extend(chunk_vertices.vertices());
    indices.extend(chunk_vertices.indices());
    draws.push(LodDraw { indices: index_offset..index_offset + chunk_vertices.indices().len() as u32, base_vertex: vertex_offset });
  }
}
