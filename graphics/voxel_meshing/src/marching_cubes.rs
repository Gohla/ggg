///! Marching cubes implementation based on:
///!
///! * https://www.youtube.com/watch?v=vTMEdHcKgM4
///! * https://www.youtube.com/watch?v=M3iI2l0ltbE / https://github.com/SebLague/Marching-Cubes
///! * https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu
///! * http://paulbourke.net/geometry/polygonise/
///! * https://people.eecs.berkeley.edu/~jrs/meshpapers/LorensenCline.pdf
///! * https://www.boristhebrave.com/2018/04/15/marching-cubes-tutorial/
///! * https://github.com/swiftcoder/isosurface

use ultraviolet::{UVec3, Vec3};

use crate::vertex::Vertex;
use crate::volume::Volume;

#[derive(Copy, Clone)]
pub struct MarchingCubes {
  surface_level: f32,
}

#[derive(Copy, Clone)]
pub struct MarchingCubesSettings {
  pub surface_level: f32,
}

impl Default for MarchingCubesSettings {
  fn default() -> Self {
    Self { surface_level: 0.0 }
  }
}

impl MarchingCubes {
  pub fn new(settings: MarchingCubesSettings) -> Self {
    Self { surface_level: settings.surface_level }
  }

  pub fn generate_into<V: Volume>(&self, start: UVec3, end: UVec3, step: u32, volume: &V, vertices: &mut Vec<Vertex>) {
    let step_usize = step as usize;
    for x in (start.x..=end.x).step_by(step_usize) {
      for y in (start.y..=end.y).step_by(step_usize) {
        for z in (start.z..=end.z).step_by(step_usize) {
          self.add_cube_vertices(UVec3::new(x, y, z), step, volume, vertices);
        }
      }
    }
  }

  #[inline]
  fn add_cube_vertices<V: Volume>(&self, pos: UVec3, step: u32, volume: &V, vertices: &mut Vec<Vertex>) {
    /* v5.+------+v6
       .' |    .'|
    v1+---+--+'v2|
      |   |  |   |
    +y|v4,+--+---+v7
      |.' +x | .'+z
    v0+------+'v3 */
    let local_vertices = [
      pos + UVec3::new(0, 0, 0), // v0
      pos + UVec3::new(0, step, 0), // v1
      pos + UVec3::new(step, step, 0), // v2
      pos + UVec3::new(step, 0, 0), // v3
      pos + UVec3::new(0, 0, step), // v4
      pos + UVec3::new(0, step, step), // v5
      pos + UVec3::new(step, step, step), // v6
      pos + UVec3::new(step, 0, step), // v7
    ];

    let mut configuration = 0;
    for (i, local_vertex) in local_vertices.iter().enumerate() {
      let value = volume.sample(local_vertex);
      if value < self.surface_level {
        configuration |= 1 << i;
      }
    }

    let edge_indices: &EdgeIndices = &TRIANGULATION[configuration];
    for i in (0..16).step_by(3) {
      if edge_indices[i] == N { break; }
      let a_0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 0] as usize] as usize;
      let a_1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 0] as usize] as usize;
      let b_0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 1] as usize] as usize;
      let b_1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 1] as usize] as usize;
      let c_0 = CORNER_INDEX_A_FROM_EDGE[edge_indices[i + 2] as usize] as usize;
      let c_1 = CORNER_INDEX_B_FROM_EDGE[edge_indices[i + 2] as usize] as usize;
      let pos_a = self.vertex_position(volume, local_vertices[a_0], local_vertices[a_1]);
      let pos_b = self.vertex_position(volume, local_vertices[b_0], local_vertices[b_1]);
      let pos_c = self.vertex_position(volume, local_vertices[c_0], local_vertices[c_1]);
      let normal = (pos_a - pos_b).cross(pos_c - pos_b).normalized();
      vertices.push(Vertex::new(pos_a, normal));
      vertices.push(Vertex::new(pos_b, normal));
      vertices.push(Vertex::new(pos_c, normal));
    }
  }

  #[inline]
  fn vertex_position<V: Volume>(&self, volume: &V, pos_a: UVec3, pos_b: UVec3) -> Vec3 {
    let value_a = volume.sample(&pos_a);
    let value_b = volume.sample(&pos_b);
    let t = (self.surface_level - value_a) / (value_b - value_a);
    let pos_a = Vec3::from(pos_a);
    let pos_b = Vec3::from(pos_b);
    pos_a + t * (pos_b - pos_a)
  }
}


type Edge = u8;
type EdgeIndices = [Edge; 16];

/// Value for no index.
const N: Edge = Edge::MAX;

/// 2D triangulation lookup table that goes from configuration (bitwise concatenation of vertex
/// indices) to array of edge indices.
///
/// The first index is the configuration. Since a cube has 8 corners, there are 2^8 = 256 entries.
///
/// The nested array consist of edge indices used to form triangles. Therefore, these always come
/// in pairs of three. No configuration spans more than 15 edges. The value `N` indicates that there
/// are no further edges for this configuration. Every array always ends with one `N` value and
/// therefore always have size 16.
///
/// From: http://paulbourke.net/geometry/polygonise/ and https://www.youtube.com/watch?v=vTMEdHcKgM4
const TRIANGULATION: [EdgeIndices; 256] = [
  [N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [1, 8, 3, 9, 8, 1, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 10, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 1, 2, 10, N, N, N, N, N, N, N, N, N, N],
  [9, 2, 10, 0, 2, 9, N, N, N, N, N, N, N, N, N, N],
  [2, 8, 3, 2, 10, 8, 10, 9, 8, N, N, N, N, N, N, N],
  [3, 11, 2, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 11, 2, 8, 11, 0, N, N, N, N, N, N, N, N, N, N],
  [1, 9, 0, 2, 3, 11, N, N, N, N, N, N, N, N, N, N],
  [1, 11, 2, 1, 9, 11, 9, 8, 11, N, N, N, N, N, N, N],
  [3, 10, 1, 11, 10, 3, N, N, N, N, N, N, N, N, N, N],
  [0, 10, 1, 0, 8, 10, 8, 11, 10, N, N, N, N, N, N, N],
  [3, 9, 0, 3, 11, 9, 11, 10, 9, N, N, N, N, N, N, N],
  [9, 8, 10, 10, 8, 11, N, N, N, N, N, N, N, N, N, N],
  [4, 7, 8, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [4, 3, 0, 7, 3, 4, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, 8, 4, 7, N, N, N, N, N, N, N, N, N, N],
  [4, 1, 9, 4, 7, 1, 7, 3, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 8, 4, 7, N, N, N, N, N, N, N, N, N, N],
  [3, 4, 7, 3, 0, 4, 1, 2, 10, N, N, N, N, N, N, N],
  [9, 2, 10, 9, 0, 2, 8, 4, 7, N, N, N, N, N, N, N],
  [2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, N, N, N, N],
  [8, 4, 7, 3, 11, 2, N, N, N, N, N, N, N, N, N, N],
  [11, 4, 7, 11, 2, 4, 2, 0, 4, N, N, N, N, N, N, N],
  [9, 0, 1, 8, 4, 7, 2, 3, 11, N, N, N, N, N, N, N],
  [4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, N, N, N, N],
  [3, 10, 1, 3, 11, 10, 7, 8, 4, N, N, N, N, N, N, N],
  [1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, N, N, N, N],
  [4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, N, N, N, N],
  [4, 7, 11, 4, 11, 9, 9, 11, 10, N, N, N, N, N, N, N],
  [9, 5, 4, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [9, 5, 4, 0, 8, 3, N, N, N, N, N, N, N, N, N, N],
  [0, 5, 4, 1, 5, 0, N, N, N, N, N, N, N, N, N, N],
  [8, 5, 4, 8, 3, 5, 3, 1, 5, N, N, N, N, N, N, N],
  [1, 2, 10, 9, 5, 4, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 8, 1, 2, 10, 4, 9, 5, N, N, N, N, N, N, N],
  [5, 2, 10, 5, 4, 2, 4, 0, 2, N, N, N, N, N, N, N],
  [2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, N, N, N, N],
  [9, 5, 4, 2, 3, 11, N, N, N, N, N, N, N, N, N, N],
  [0, 11, 2, 0, 8, 11, 4, 9, 5, N, N, N, N, N, N, N],
  [0, 5, 4, 0, 1, 5, 2, 3, 11, N, N, N, N, N, N, N],
  [2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, N, N, N, N],
  [10, 3, 11, 10, 1, 3, 9, 5, 4, N, N, N, N, N, N, N],
  [4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, N, N, N, N],
  [5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, N, N, N, N],
  [5, 4, 8, 5, 8, 10, 10, 8, 11, N, N, N, N, N, N, N],
  [9, 7, 8, 5, 7, 9, N, N, N, N, N, N, N, N, N, N],
  [9, 3, 0, 9, 5, 3, 5, 7, 3, N, N, N, N, N, N, N],
  [0, 7, 8, 0, 1, 7, 1, 5, 7, N, N, N, N, N, N, N],
  [1, 5, 3, 3, 5, 7, N, N, N, N, N, N, N, N, N, N],
  [9, 7, 8, 9, 5, 7, 10, 1, 2, N, N, N, N, N, N, N],
  [10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, N, N, N, N],
  [8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, N, N, N, N],
  [2, 10, 5, 2, 5, 3, 3, 5, 7, N, N, N, N, N, N, N],
  [7, 9, 5, 7, 8, 9, 3, 11, 2, N, N, N, N, N, N, N],
  [9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, N, N, N, N],
  [2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, N, N, N, N],
  [11, 2, 1, 11, 1, 7, 7, 1, 5, N, N, N, N, N, N, N],
  [9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, N, N, N, N],
  [5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, N],
  [11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, N],
  [11, 10, 5, 7, 11, 5, N, N, N, N, N, N, N, N, N, N],
  [10, 6, 5, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 5, 10, 6, N, N, N, N, N, N, N, N, N, N],
  [9, 0, 1, 5, 10, 6, N, N, N, N, N, N, N, N, N, N],
  [1, 8, 3, 1, 9, 8, 5, 10, 6, N, N, N, N, N, N, N],
  [1, 6, 5, 2, 6, 1, N, N, N, N, N, N, N, N, N, N],
  [1, 6, 5, 1, 2, 6, 3, 0, 8, N, N, N, N, N, N, N],
  [9, 6, 5, 9, 0, 6, 0, 2, 6, N, N, N, N, N, N, N],
  [5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, N, N, N, N],
  [2, 3, 11, 10, 6, 5, N, N, N, N, N, N, N, N, N, N],
  [11, 0, 8, 11, 2, 0, 10, 6, 5, N, N, N, N, N, N, N],
  [0, 1, 9, 2, 3, 11, 5, 10, 6, N, N, N, N, N, N, N],
  [5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, N, N, N, N],
  [6, 3, 11, 6, 5, 3, 5, 1, 3, N, N, N, N, N, N, N],
  [0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, N, N, N, N],
  [3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, N, N, N, N],
  [6, 5, 9, 6, 9, 11, 11, 9, 8, N, N, N, N, N, N, N],
  [5, 10, 6, 4, 7, 8, N, N, N, N, N, N, N, N, N, N],
  [4, 3, 0, 4, 7, 3, 6, 5, 10, N, N, N, N, N, N, N],
  [1, 9, 0, 5, 10, 6, 8, 4, 7, N, N, N, N, N, N, N],
  [10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, N, N, N, N],
  [6, 1, 2, 6, 5, 1, 4, 7, 8, N, N, N, N, N, N, N],
  [1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, N, N, N, N],
  [8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, N, N, N, N],
  [7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9, N],
  [3, 11, 2, 7, 8, 4, 10, 6, 5, N, N, N, N, N, N, N],
  [5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, N, N, N, N],
  [0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, N, N, N, N],
  [9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6, N],
  [8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, N, N, N, N],
  [5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11, N],
  [0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, N],
  [6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, N, N, N, N],
  [10, 4, 9, 6, 4, 10, N, N, N, N, N, N, N, N, N, N],
  [4, 10, 6, 4, 9, 10, 0, 8, 3, N, N, N, N, N, N, N],
  [10, 0, 1, 10, 6, 0, 6, 4, 0, N, N, N, N, N, N, N],
  [8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, N, N, N, N],
  [1, 4, 9, 1, 2, 4, 2, 6, 4, N, N, N, N, N, N, N],
  [3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, N, N, N, N],
  [0, 2, 4, 4, 2, 6, N, N, N, N, N, N, N, N, N, N],
  [8, 3, 2, 8, 2, 4, 4, 2, 6, N, N, N, N, N, N, N],
  [10, 4, 9, 10, 6, 4, 11, 2, 3, N, N, N, N, N, N, N],
  [0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, N, N, N, N],
  [3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, N, N, N, N],
  [6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, N],
  [9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, N, N, N, N],
  [8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, N],
  [3, 11, 6, 3, 6, 0, 0, 6, 4, N, N, N, N, N, N, N],
  [6, 4, 8, 11, 6, 8, N, N, N, N, N, N, N, N, N, N],
  [7, 10, 6, 7, 8, 10, 8, 9, 10, N, N, N, N, N, N, N],
  [0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, N, N, N, N],
  [10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, N, N, N, N],
  [10, 6, 7, 10, 7, 1, 1, 7, 3, N, N, N, N, N, N, N],
  [1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, N, N, N, N],
  [2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9, N],
  [7, 8, 0, 7, 0, 6, 6, 0, 2, N, N, N, N, N, N, N],
  [7, 3, 2, 6, 7, 2, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, N, N, N, N],
  [2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, N],
  [1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, N],
  [11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, N, N, N, N],
  [8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, N],
  [0, 9, 1, 11, 6, 7, N, N, N, N, N, N, N, N, N, N],
  [7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, N, N, N, N],
  [7, 11, 6, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [7, 6, 11, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 8, 11, 7, 6, N, N, N, N, N, N, N, N, N, N],
  [0, 1, 9, 11, 7, 6, N, N, N, N, N, N, N, N, N, N],
  [8, 1, 9, 8, 3, 1, 11, 7, 6, N, N, N, N, N, N, N],
  [10, 1, 2, 6, 11, 7, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 10, 3, 0, 8, 6, 11, 7, N, N, N, N, N, N, N],
  [2, 9, 0, 2, 10, 9, 6, 11, 7, N, N, N, N, N, N, N],
  [6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, N, N, N, N],
  [7, 2, 3, 6, 2, 7, N, N, N, N, N, N, N, N, N, N],
  [7, 0, 8, 7, 6, 0, 6, 2, 0, N, N, N, N, N, N, N],
  [2, 7, 6, 2, 3, 7, 0, 1, 9, N, N, N, N, N, N, N],
  [1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, N, N, N, N],
  [10, 7, 6, 10, 1, 7, 1, 3, 7, N, N, N, N, N, N, N],
  [10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, N, N, N, N],
  [0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, N, N, N, N],
  [7, 6, 10, 7, 10, 8, 8, 10, 9, N, N, N, N, N, N, N],
  [6, 8, 4, 11, 8, 6, N, N, N, N, N, N, N, N, N, N],
  [3, 6, 11, 3, 0, 6, 0, 4, 6, N, N, N, N, N, N, N],
  [8, 6, 11, 8, 4, 6, 9, 0, 1, N, N, N, N, N, N, N],
  [9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, N, N, N, N],
  [6, 8, 4, 6, 11, 8, 2, 10, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, N, N, N, N],
  [4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, N, N, N, N],
  [10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, N],
  [8, 2, 3, 8, 4, 2, 4, 6, 2, N, N, N, N, N, N, N],
  [0, 4, 2, 4, 6, 2, N, N, N, N, N, N, N, N, N, N],
  [1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, N, N, N, N],
  [1, 9, 4, 1, 4, 2, 2, 4, 6, N, N, N, N, N, N, N],
  [8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, N, N, N, N],
  [10, 1, 0, 10, 0, 6, 6, 0, 4, N, N, N, N, N, N, N],
  [4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, N],
  [10, 9, 4, 6, 10, 4, N, N, N, N, N, N, N, N, N, N],
  [4, 9, 5, 7, 6, 11, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 3, 4, 9, 5, 11, 7, 6, N, N, N, N, N, N, N],
  [5, 0, 1, 5, 4, 0, 7, 6, 11, N, N, N, N, N, N, N],
  [11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, N, N, N, N],
  [9, 5, 4, 10, 1, 2, 7, 6, 11, N, N, N, N, N, N, N],
  [6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, N, N, N, N],
  [7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, N, N, N, N],
  [3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, N],
  [7, 2, 3, 7, 6, 2, 5, 4, 9, N, N, N, N, N, N, N],
  [9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, N, N, N, N],
  [3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, N, N, N, N],
  [6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, N],
  [9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, N, N, N, N],
  [1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, N],
  [4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, N],
  [7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, N, N, N, N],
  [6, 9, 5, 6, 11, 9, 11, 8, 9, N, N, N, N, N, N, N],
  [3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, N, N, N, N],
  [0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, N, N, N, N],
  [6, 11, 3, 6, 3, 5, 5, 3, 1, N, N, N, N, N, N, N],
  [1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, N, N, N, N],
  [0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, N],
  [11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5, N],
  [6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, N, N, N, N],
  [5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, N, N, N, N],
  [9, 5, 6, 9, 6, 0, 0, 6, 2, N, N, N, N, N, N, N],
  [1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8, N],
  [1, 5, 6, 2, 1, 6, N, N, N, N, N, N, N, N, N, N],
  [1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6, N],
  [10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, N, N, N, N],
  [0, 3, 8, 5, 6, 10, N, N, N, N, N, N, N, N, N, N],
  [10, 5, 6, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [11, 5, 10, 7, 5, 11, N, N, N, N, N, N, N, N, N, N],
  [11, 5, 10, 11, 7, 5, 8, 3, 0, N, N, N, N, N, N, N],
  [5, 11, 7, 5, 10, 11, 1, 9, 0, N, N, N, N, N, N, N],
  [10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, N, N, N, N],
  [11, 1, 2, 11, 7, 1, 7, 5, 1, N, N, N, N, N, N, N],
  [0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, N, N, N, N],
  [9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, N, N, N, N],
  [7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, N],
  [2, 5, 10, 2, 3, 5, 3, 7, 5, N, N, N, N, N, N, N],
  [8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, N, N, N, N],
  [9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, N, N, N, N],
  [9, 8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2, N],
  [1, 3, 5, 3, 7, 5, N, N, N, N, N, N, N, N, N, N],
  [0, 8, 7, 0, 7, 1, 1, 7, 5, N, N, N, N, N, N, N],
  [9, 0, 3, 9, 3, 5, 5, 3, 7, N, N, N, N, N, N, N],
  [9, 8, 7, 5, 9, 7, N, N, N, N, N, N, N, N, N, N],
  [5, 8, 4, 5, 10, 8, 10, 11, 8, N, N, N, N, N, N, N],
  [5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, N, N, N, N],
  [0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, N, N, N, N],
  [10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, N],
  [2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, N, N, N, N],
  [0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, N],
  [0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, N],
  [9, 4, 5, 2, 11, 3, N, N, N, N, N, N, N, N, N, N],
  [2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, N, N, N, N],
  [5, 10, 2, 5, 2, 4, 4, 2, 0, N, N, N, N, N, N, N],
  [3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, N],
  [5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, N, N, N, N],
  [8, 4, 5, 8, 5, 3, 3, 5, 1, N, N, N, N, N, N, N],
  [0, 4, 5, 1, 0, 5, N, N, N, N, N, N, N, N, N, N],
  [8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, N, N, N, N],
  [9, 4, 5, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [4, 11, 7, 4, 9, 11, 9, 10, 11, N, N, N, N, N, N, N],
  [0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, N, N, N, N],
  [1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, N, N, N, N],
  [3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, N],
  [4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, N, N, N, N],
  [9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, N],
  [11, 7, 4, 11, 4, 2, 2, 4, 0, N, N, N, N, N, N, N],
  [11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, N, N, N, N],
  [2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, N, N, N, N],
  [9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, N],
  [3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10, N],
  [1, 10, 2, 8, 7, 4, N, N, N, N, N, N, N, N, N, N],
  [4, 9, 1, 4, 1, 7, 7, 1, 3, N, N, N, N, N, N, N],
  [4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, N, N, N, N],
  [4, 0, 3, 7, 4, 3, N, N, N, N, N, N, N, N, N, N],
  [4, 8, 7, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [9, 10, 8, 10, 11, 8, N, N, N, N, N, N, N, N, N, N],
  [3, 0, 9, 3, 9, 11, 11, 9, 10, N, N, N, N, N, N, N],
  [0, 1, 10, 0, 10, 8, 8, 10, 11, N, N, N, N, N, N, N],
  [3, 1, 10, 11, 3, 10, N, N, N, N, N, N, N, N, N, N],
  [1, 2, 11, 1, 11, 9, 9, 11, 8, N, N, N, N, N, N, N],
  [3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, N, N, N, N],
  [0, 2, 11, 8, 0, 11, N, N, N, N, N, N, N, N, N, N],
  [3, 2, 11, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 8, 2, 8, 10, 10, 8, 9, N, N, N, N, N, N, N],
  [9, 10, 2, 0, 9, 2, N, N, N, N, N, N, N, N, N, N],
  [2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, N, N, N, N],
  [1, 10, 2, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [1, 3, 8, 9, 1, 8, N, N, N, N, N, N, N, N, N, N],
  [0, 9, 1, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [0, 3, 8, N, N, N, N, N, N, N, N, N, N, N, N, N],
  [N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N]
];

const CORNER_INDEX_A_FROM_EDGE: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3];
const CORNER_INDEX_B_FROM_EDGE: [u8; 12] = [1, 2, 3, 0, 5, 6, 7, 4, 4, 5, 6, 7];