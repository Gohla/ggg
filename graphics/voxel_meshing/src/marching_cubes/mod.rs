use std::marker::PhantomData;

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

use crate::chunk::{ChunkSampleArray, ChunkSamples, ChunkSize, ChunkVertices, Vertex};
use crate::marching_cubes::tables::VertexData;

pub mod tables;

#[derive(Copy, Clone)]
pub struct MarchingCubes<C: ChunkSize> {
  _chunk_phantom: PhantomData<C>,
}

impl<C: ChunkSize> MarchingCubes<C> {
  pub const SHARED_INDICES_SIZE: usize = 4 * C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE;

  pub fn new() -> Self {
    Self { _chunk_phantom: PhantomData::default() }
  }

  pub fn extract_chunk(
    &self,
    min: UVec3,
    step: u32,
    chunk_samples: &ChunkSamples<C>,
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:,
    [u16; Self::SHARED_INDICES_SIZE]:,
  {
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      let mut shared_indices = [u16::MAX; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
      for cell_y in 0..C::CELLS_IN_CHUNK_ROW { // NOTE: Y and Z axis flipped!
        for cell_z in 0..C::CELLS_IN_CHUNK_ROW { // NOTE: Z and Y axis flipped!
          for cell_x in 0..C::CELLS_IN_CHUNK_ROW {
            let cell = UVec3::new(cell_x, cell_y, cell_z);
            Self::extract_cell(cell, min, step, chunk_sample_array, &mut shared_indices, chunk_vertices);
          }
        }
      }
    }
  }

  #[inline]
  fn extract_cell(
    cell: UVec3,
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    let local_voxels = Self::local_voxels(cell);
    let values = Self::sample(chunk_sample_array, &local_voxels);
    let case = Self::case(&values);
    if case == 0 || case == 255 { // No triangles // OPTO: use bit twiddling to break it down to 1 comparison?
      return;
    }
    let case = case as usize;

    // Get the cell class for the `case`.
    let cell_class = tables::CELL_CLASS[case];
    // Get the triangulation info corresponding to the cell class. This uses `cell_class` instead of `case`, because the
    // triangulation info is equivalent for a class of cells. The full `case` is used along with this info to form the
    // eventual triangles.
    let triangulation_info = tables::CELL_DATA[cell_class as usize];
    let vertex_count = triangulation_info.get_vertex_count() as usize;
    let triangle_count = triangulation_info.get_triangle_count();
    // Get the vertex data corresponding to the `case`.
    let vertices_data = tables::VERTEX_DATA[case];

    // Get global voxels to put vertices on the correct location.
    let global_voxels = Self::global_voxels(min, step, &local_voxels);
    // Get indices for all vertices, creating new vertices and thus new indices, or reusing indices from previous cells.
    let mut cell_vertices_indices = [0; 12];
    for (i, vd) in vertices_data.iter().enumerate() {
      if i >= vertex_count {
        break;
      }
      let index = Self::create_or_reuse_vertex(VertexData(*vd), cell, &global_voxels, &values, shared_indices, chunk_vertices);
      cell_vertices_indices[i] = index;
    }

    // Write the indices that form the final triangulation of this cell.
    for t in 0..triangle_count {
      let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
      let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
      let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
      let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
      let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
      let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
      chunk_vertices.push_index(global_index_1);
      chunk_vertices.push_index(global_index_2);
      chunk_vertices.push_index(global_index_3);
    }
  }

  pub const VOXELS: [UVec3; 8] = [ // NOTE: Y and Z axis flipped!
    UVec3::new(0, 0, 0), // 0 (0, 0, 0)
    UVec3::new(1, 0, 0), // 1 (1, 0, 0)
    UVec3::new(0, 0, 1), // 2 (0, 1, 0)
    UVec3::new(1, 0, 1), // 3 (1, 1, 0)
    UVec3::new(0, 1, 0), // 4 (0, 0, 1)
    UVec3::new(1, 1, 0), // 5 (1, 0, 1)
    UVec3::new(0, 1, 1), // 6 (0, 1, 1)
    UVec3::new(1, 1, 1), // 7 (1, 1, 1)
  ];

  #[inline(always)]
  pub fn local_voxels(cell: UVec3) -> [UVec3; 8] {
    [
      cell + Self::VOXELS[0],
      cell + Self::VOXELS[1],
      cell + Self::VOXELS[2],
      cell + Self::VOXELS[3],
      cell + Self::VOXELS[4],
      cell + Self::VOXELS[5],
      cell + Self::VOXELS[6],
      cell + Self::VOXELS[7],
    ]
  }

  #[inline(always)]
  pub fn global_voxels(min: UVec3, step: u32, local_voxels: &[UVec3; 8]) -> [UVec3; 8] {
    [
      min + step * local_voxels[0],
      min + step * local_voxels[1],
      min + step * local_voxels[2],
      min + step * local_voxels[3],
      min + step * local_voxels[4],
      min + step * local_voxels[5],
      min + step * local_voxels[6],
      min + step * local_voxels[7],
    ]
  }

  #[inline(always)]
  pub fn sample(chunk_sample_array: &ChunkSampleArray<C>, local_voxels: &[UVec3; 8]) -> [f32; 8] where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    [
      chunk_sample_array.sample(local_voxels[0]),
      chunk_sample_array.sample(local_voxels[1]),
      chunk_sample_array.sample(local_voxels[2]),
      chunk_sample_array.sample(local_voxels[3]),
      chunk_sample_array.sample(local_voxels[4]),
      chunk_sample_array.sample(local_voxels[5]),
      chunk_sample_array.sample(local_voxels[6]),
      chunk_sample_array.sample(local_voxels[7]),
    ]
  }

  #[inline(always)]
  pub fn case(values: &[f32; 8]) -> u8 {
    // Create the case number by packing the sign bits of samples. Negative = inside, positive = outside.
    (values[0].is_sign_negative() as u8) << 0
      | (values[1].is_sign_negative() as u8) << 1
      | (values[2].is_sign_negative() as u8) << 2
      | (values[3].is_sign_negative() as u8) << 3
      | (values[4].is_sign_negative() as u8) << 4
      | (values[5].is_sign_negative() as u8) << 5
      | (values[6].is_sign_negative() as u8) << 6
      | (values[7].is_sign_negative() as u8) << 7
  }

  #[inline]
  fn create_or_reuse_vertex(
    vertex_data: VertexData,
    cell: UVec3,
    global_voxels: &[UVec3; 8],
    values: &[f32; 8],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
  ) -> u16 {
    if vertex_data.new_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_vertices);
      let shared_indices_index = Self::shared_index(cell, vertex_data.vertex_index() as usize);
      debug_assert!(shared_indices_index < shared_indices.len(), "Tried to write out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      debug_assert!(shared_indices[shared_indices_index] == u16::MAX, "Tried to write already set shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      shared_indices[shared_indices_index] = index;
      index
    } else {
      let subtract_x = vertex_data.subtract_x();
      let subtract_y = vertex_data.subtract_y();
      let subtract_z = vertex_data.subtract_z();
      // OPTO: use 3-bit validity mask as proposed in the paper?
      if (cell.x > 0 || !subtract_x) && (cell.y > 0 || !subtract_y) && (cell.z > 0 || !subtract_z) {
        // Return index of previous vertex.
        let previous_cell = {
          let mut previous_cell = cell;
          if subtract_x { previous_cell.x -= 1; }
          if subtract_y { previous_cell.y -= 1; }
          if subtract_z { previous_cell.z -= 1; }
          previous_cell
        };
        let shared_indices_index = Self::shared_index(previous_cell, vertex_data.vertex_index() as usize);
        debug_assert!(shared_indices_index < shared_indices.len(), "Tried to read out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell);
        let index = shared_indices[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_vertices);
        index
      }
    }
  }

  #[inline]
  pub fn create_vertex(
    vertex_data: VertexData,
    global_voxels: &[UVec3; 8],
    values: &[f32; 8],
    chunk_vertices: &mut ChunkVertices,
  ) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    debug_assert!(voxel_b_index > voxel_a_index, "Voxel B index {} is lower than voxel A index {}, which leads to inconsistencies", voxel_b_index, voxel_a_index);
    let pos_low = global_voxels[voxel_a_index as usize];
    let value_low = values[voxel_a_index as usize];
    let pos_high = global_voxels[voxel_b_index as usize];
    let value_high = values[voxel_b_index as usize];
    let position = Self::vertex_position(pos_low, value_low, pos_high, value_high);
    chunk_vertices.push_vertex(Vertex::new(position))
  }

  #[inline(always)]
  pub fn vertex_position(pos_low: UVec3, value_low: f32, pos_high: UVec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    let pos_low = Vec3::from(pos_low);
    let pos_high = Vec3::from(pos_high);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline(always)]
  pub fn shared_index(cell: UVec3, vertex_index: usize) -> usize {
    cell.x as usize
      + C::CELLS_IN_CHUNK_ROW_USIZE * cell.y as usize
      + C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE * cell.z as usize
      + C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE * vertex_index
  }
}
