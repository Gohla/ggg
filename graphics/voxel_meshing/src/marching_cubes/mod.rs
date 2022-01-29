///! Marching cubes implementation based on:
///!
///! * https://www.youtube.com/watch?v=vTMEdHcKgM4
///! * https://www.youtube.com/watch?v=M3iI2l0ltbE / https://github.com/SebLague/Marching-Cubes
///! * https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu
///! * http://paulbourke.net/geometry/polygonise/
///! * https://people.eecs.berkeley.edu/~jrs/meshpapers/LorensenCline.pdf
///! * https://www.boristhebrave.com/2018/04/15/marching-cubes-tutorial/
///! * https://github.com/swiftcoder/isosurface
///! * https://transvoxel.org/
///! * https://docs.rs/transvoxel/latest/transvoxel/

use ultraviolet::{UVec3, Vec3};

use crate::{CHUNK_SIZE, CHUNK_SIZE_USIZE};
use crate::chunk::{Chunk, Vertex};
use crate::volume::Volume;

mod tables;

#[derive(Copy, Clone)]
pub struct MarchingCubes;


impl MarchingCubes {
  const SHARED_INDICES_SIZE: usize = 4 * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;

  pub fn extract_chunk<V: Volume>(&self, start: UVec3, step: u32, volume: &V, chunk: &mut Chunk) {
    let mut shared_indices = [0; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
    for cell_z in 0..CHUNK_SIZE {
      for cell_y in 0..CHUNK_SIZE {
        for cell_x in 0..CHUNK_SIZE {
          let cell_local = UVec3::new(cell_x, cell_y, cell_z);
          let cell_global = start + step * cell_local;
          Self::extract_cell(cell_local, cell_global, step, volume, &mut shared_indices, chunk);
        }
      }
    }
  }

  #[inline]
  fn extract_cell<V: Volume>(
    cell_local: UVec3,
    cell_global: UVec3,
    step: u32,
    volume: &V,
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) {
    // Get voxels of the cell.
    let zero = 0;
    let voxels = [
      cell_global + UVec3::new(zero, zero, zero), // 0
      cell_global + UVec3::new(step, zero, zero), // 1
      cell_global + UVec3::new(zero, step, zero), // 2
      cell_global + UVec3::new(step, step, zero), // 3
      cell_global + UVec3::new(zero, zero, step), // 4
      cell_global + UVec3::new(step, zero, step), // 5
      cell_global + UVec3::new(zero, step, step), // 6
      cell_global + UVec3::new(step, step, step), // 7
    ];
    // Sample the volume at each voxel, producing values.
    let values = [
      volume.sample(voxels[0]),
      volume.sample(voxels[1]),
      volume.sample(voxels[2]),
      volume.sample(voxels[3]),
      volume.sample(voxels[4]),
      volume.sample(voxels[5]),
      volume.sample(voxels[6]),
      volume.sample(voxels[7]),
    ];
    // Create the case number by packing the sign bits of samples. Positive = inside.
    let case = values[0].is_sign_positive() as u8 >> 7
      | values[1].is_sign_positive() as u8 >> 6
      | values[2].is_sign_positive() as u8 >> 5
      | values[3].is_sign_positive() as u8 >> 4
      | values[4].is_sign_positive() as u8 >> 3
      | values[5].is_sign_positive() as u8 >> 2
      | values[6].is_sign_positive() as u8 >> 1
      | values[7].is_sign_positive() as u8;
    if case == 0 || case == 255 { // No triangles // OPTO: use bit twiddling to break it down to 1 comparison?
      return;
    }

    // Get the cell class for the `case`.
    let cell_class = tables::REGULAR_CELL_CLASS[case as usize];
    // Get the triangulation info corresponding to the cell class. This uses `cell_class` instead of `case`, because the
    // triangulation info is equivalent for a class of cells. The full `case` is used along with this info to form the
    // eventual triangles.
    let triangulation_info = tables::REGULAR_CELL_DATA[cell_class as usize];
    // Get the vertex data corresponding to the `case`.
    let vertices_data = tables::REGULAR_VERTEX_DATA[case as usize];

    let mut cell_vertices_indices = [0; 12];
    for (i, vd) in vertices_data.iter().enumerate() {
      if i >= triangulation_info.get_vertex_count() as usize {
        break;
      }
      let vertex_data = tables::RegularVertexData(*vd);
      let index = Self::create_or_reuse_vertex(vertex_data, cell_local, &voxels, &values, shared_indices, chunk);
      cell_vertices_indices[i] = index;
    }

    for t in 0..triangulation_info.get_triangle_count() {
      let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
      let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
      let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
      let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
      let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
      let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
      chunk.indices.push(global_index_1);
      chunk.indices.push(global_index_2);
      chunk.indices.push(global_index_3);
    }
  }

  #[inline]
  fn create_or_reuse_vertex(
    vertex_data: tables::RegularVertexData,
    cell_local: UVec3,
    voxels: &[UVec3; 8],
    values: &[f32; 8],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) -> u16 {
    if vertex_data.new_vertex() { // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, voxels, values, chunk);
      let shared_indices_index = Self::shared_index(cell_local, vertex_data.reuse_index() as usize);
      shared_indices[shared_indices_index] = index;
      index
    } else { // TODO: use 3-bit validity mask as proposed in the paper?
      let previous_vertex_is_accessible = ((vertex_data.reuse_dx() == 0) || (cell_local.x > 0))
        && ((vertex_data.reuse_dy() == 0) || (cell_local.y > 0))
        && ((vertex_data.reuse_dz() == 0) || (cell_local.z > 0));
      if previous_vertex_is_accessible { // Return index of previous vertex.
        let previous_cell_local = {
          let mut previous_cell_local = cell_local;
          previous_cell_local.x += (-vertex_data.reuse_dx()) as u32; // TODO: make this less crappy.
          previous_cell_local.y += (-vertex_data.reuse_dy()) as u32;
          previous_cell_local.z += (-vertex_data.reuse_dz()) as u32;
          previous_cell_local
        };
        let shared_indices_index = Self::shared_index(previous_cell_local, vertex_data.reuse_index() as usize);
        shared_indices[shared_indices_index]
      } else { // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, voxels, values, chunk);
        index
      }
    }
  }

  #[inline]
  fn create_vertex(vertex_data: tables::RegularVertexData, voxels: &[UVec3; 8], values: &[f32; 8], chunk: &mut Chunk) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    let vertex = Self::vertex_position(voxels[voxel_a_index as usize], values[voxel_a_index as usize], voxels[voxel_b_index as usize], values[voxel_b_index as usize]);
    chunk.vertices.push(Vertex::new(vertex, Vec3::one())); // TODO: calculate normal!
    let index = chunk.vertices.len() as u16;
    index
  }

  #[inline]
  fn vertex_position(pos_a: UVec3, value_a: f32, pos_b: UVec3, value_b: f32) -> Vec3 {
    let t = value_b / (value_b - value_a);
    let pos_a = Vec3::from(pos_a);
    let pos_b = Vec3::from(pos_b);
    t * pos_a + (1.0 - t) * pos_b
  }

  #[inline]
  fn shared_index(cell_local: UVec3, reuse_index: usize) -> usize {
    cell_local.x as usize
      + CHUNK_SIZE_USIZE * cell_local.y as usize
      + CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * cell_local.z as usize
      + CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * reuse_index
  }
}
