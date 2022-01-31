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
    let mut shared_indices = [u16::MAX; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
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
    let case = (values[0].is_sign_positive() as u8) << 0
      | (values[1].is_sign_positive() as u8) << 1
      | (values[2].is_sign_positive() as u8) << 2
      | (values[3].is_sign_positive() as u8) << 3
      | (values[4].is_sign_positive() as u8) << 4
      | (values[5].is_sign_positive() as u8) << 5
      | (values[6].is_sign_positive() as u8) << 6
      | (values[7].is_sign_positive() as u8) << 7;
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
    if vertex_data.new_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, voxels, values, chunk);
      let shared_indices_index = Self::shared_index(cell_local, vertex_data.vertex_index() as usize);
      debug_assert!(shared_indices_index < shared_indices.len(), "Tried to write out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, cell_local);
      debug_assert!(shared_indices[shared_indices_index] == u16::MAX, "Tried to write already set shared index, at index: {}, position: {:?}", shared_indices_index, cell_local);
      shared_indices[shared_indices_index] = index;
      index
    } else {
      let subtract_x = vertex_data.subtract_x();
      let subtract_y = vertex_data.subtract_y();
      let subtract_z = vertex_data.subtract_z();
      // OPTO: use 3-bit validity mask as proposed in the paper?
      if (cell_local.x > 0 || !subtract_x) && (cell_local.y > 0 || !subtract_y) && (cell_local.z > 0 || !subtract_z) {
        // Return index of previous vertex.
        let previous_cell_local = {
          let mut previous_cell_local = cell_local;
          if subtract_x { previous_cell_local.x -= 1; }
          if subtract_y { previous_cell_local.y -= 1; }
          if subtract_z { previous_cell_local.z -= 1; }
          previous_cell_local
        };
        let shared_indices_index = Self::shared_index(previous_cell_local, vertex_data.vertex_index() as usize);
        debug_assert!(shared_indices_index < shared_indices.len(), "Tried to read out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        let index = shared_indices[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, voxels, values, chunk);
        index
      }
    }
  }

  #[inline]
  fn create_vertex(vertex_data: tables::RegularVertexData, voxels: &[UVec3; 8], values: &[f32; 8], chunk: &mut Chunk) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    debug_assert!(voxel_b_index > voxel_a_index);
    let position = Self::vertex_position(voxels[voxel_a_index as usize], values[voxel_a_index as usize], voxels[voxel_b_index as usize], values[voxel_b_index as usize]);
    let index = chunk.vertices.len() as u16;
    chunk.vertices.push(Vertex::new(position));
    index
  }

  #[inline]
  fn vertex_position(pos_low: UVec3, value_low: f32, pos_high: UVec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    let pos_low = Vec3::from(pos_low);
    let pos_high = Vec3::from(pos_high);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline]
  fn shared_index(cell_local: UVec3, vertex_index: usize) -> usize {
    cell_local.x as usize
      + CHUNK_SIZE_USIZE * cell_local.y as usize
      + CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * cell_local.z as usize
      + CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * vertex_index
  }
}
