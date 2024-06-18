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

use crate::chunk::array::{Array, Slice};
use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::{ChunkSampleArray, ChunkSamples, MaybeCompressedChunkSampleArray, MaybeCompressedChunkSamples};
use crate::chunk::size::ChunkSize;
use crate::marching_cubes::tables::RegularVertexData;

pub mod tables;

#[derive(Default, Copy, Clone, Debug)]
pub struct MarchingCubes<C: ChunkSize> {
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize> MarchingCubes<C> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[profiling::function]
  pub fn extract_chunk(
    &self,
    min: UVec3,
    step: u32,
    chunk_samples: &MaybeCompressedChunkSampleArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if let MaybeCompressedChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      let mut shared_indices = C::MarchingCubesSharedIndicesArray::new(u16::MAX); // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
      for w in 0..C::CELLS_IN_CHUNK_ROW {
        for v in 0..C::CELLS_IN_CHUNK_ROW {
          for u in 0..C::CELLS_IN_CHUNK_ROW {
            let cell = RegularCell::new(u, v, w);
            Self::extract_cell(cell, min, step, chunk_sample_array, &mut shared_indices, chunk_mesh);
          }
        }
      }
    }
  }

  #[inline]
  fn extract_cell(
    cell: RegularCell,
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    shared_indices: &mut C::MarchingCubesSharedIndicesArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let local_coordinates = Self::local_coordinates(cell);
    let values = Self::sample(chunk_sample_array, &local_coordinates);
    let case = Self::case(&values);
    if case == 0 || case == 255 { // No triangles // OPTO: use bit twiddling to break it down to 1 comparison?
      return;
    }
    let case = case as usize;
    let cell_class = tables::REGULAR_CELL_CLASS[case] as usize;
    let triangulation_info = tables::REGULAR_CELL_DATA[cell_class];
    let vertex_count = triangulation_info.get_vertex_count() as usize;
    let triangle_count = triangulation_info.get_triangle_count() as usize;
    let vertices_data = tables::REGULAR_VERTEX_DATA[case];
    let global_voxels = Self::global_coordinates(min, step, &local_coordinates);
    let mut cell_vertices_indices = [0; 12];
    for (i, vd) in vertices_data[0..vertex_count].iter().enumerate() {
      let index = Self::create_or_reuse_vertex(vd, cell, &global_voxels, &values, shared_indices, chunk_mesh);
      cell_vertices_indices[i] = index;
    }
    for t in 0..triangle_count {
      let v1_index_in_cell = triangulation_info.vertex_index[3 * t];
      let v2_index_in_cell = triangulation_info.vertex_index[3 * t + 1];
      let v3_index_in_cell = triangulation_info.vertex_index[3 * t + 2];
      let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
      let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
      let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
      chunk_mesh.push_index(global_index_1);
      chunk_mesh.push_index(global_index_2);
      chunk_mesh.push_index(global_index_3);
    }
  }

  pub const VOXELS: [RegularVoxel; 8] = [
    RegularVoxel::new(0, 0, 0), // 1
    RegularVoxel::new(1, 0, 0), // 2
    RegularVoxel::new(0, 1, 0), // 3
    RegularVoxel::new(1, 1, 0), // 4
    RegularVoxel::new(0, 0, 1), // 5
    RegularVoxel::new(1, 0, 1), // 6
    RegularVoxel::new(0, 1, 1), // 7
    RegularVoxel::new(1, 1, 1), // 8
  ];

  #[inline]
  pub fn local_coordinates(cell: RegularCell) -> [UVec3; 8] {
    [
      cell.to_local_coordinate(Self::VOXELS[0]),
      cell.to_local_coordinate(Self::VOXELS[1]),
      cell.to_local_coordinate(Self::VOXELS[2]),
      cell.to_local_coordinate(Self::VOXELS[3]),
      cell.to_local_coordinate(Self::VOXELS[4]),
      cell.to_local_coordinate(Self::VOXELS[5]),
      cell.to_local_coordinate(Self::VOXELS[6]),
      cell.to_local_coordinate(Self::VOXELS[7]),
    ]
  }

  #[inline]
  pub fn sample(chunk_sample_array: &ChunkSampleArray<C>, local_coordinates: &[UVec3; 8]) -> [f32; 8] {
    [
      chunk_sample_array.sample(local_coordinates[0]),
      chunk_sample_array.sample(local_coordinates[1]),
      chunk_sample_array.sample(local_coordinates[2]),
      chunk_sample_array.sample(local_coordinates[3]),
      chunk_sample_array.sample(local_coordinates[4]),
      chunk_sample_array.sample(local_coordinates[5]),
      chunk_sample_array.sample(local_coordinates[6]),
      chunk_sample_array.sample(local_coordinates[7]),
    ]
  }

  #[inline]
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
  pub fn global_coordinates(min: UVec3, step: u32, local_coordinates: &[UVec3; 8]) -> [UVec3; 8] {
    [
      min + step * local_coordinates[0],
      min + step * local_coordinates[1],
      min + step * local_coordinates[2],
      min + step * local_coordinates[3],
      min + step * local_coordinates[4],
      min + step * local_coordinates[5],
      min + step * local_coordinates[6],
      min + step * local_coordinates[7],
    ]
  }

  #[inline]
  fn create_or_reuse_vertex(
    vertex_data: &RegularVertexData,
    cell: RegularCell,
    global_voxels: &[UVec3; 8],
    values: &[f32; 8],
    shared_indices: &mut C::MarchingCubesSharedIndicesArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) -> u16 {
    if vertex_data.new_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_mesh);
      let shared_indices_index = Self::shared_index(cell, vertex_data.vertex_index());
      debug_assert!(shared_indices.contains(shared_indices_index), "Tried to write out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      debug_assert!(shared_indices[shared_indices_index] == u16::MAX, "Tried to write already set shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      shared_indices[shared_indices_index] = index;
      index
    } else {
      let subtract_u = vertex_data.subtract_u();
      let subtract_v = vertex_data.subtract_v();
      let subtract_w = vertex_data.subtract_w();
      // OPTO: use 3-bit validity mask as proposed in the dissertation?
      if (cell.u > 0 || !subtract_u) && (cell.v > 0 || !subtract_v) && (cell.w > 0 || !subtract_w) {
        // Return index of previous vertex.
        let previous_cell = {
          let mut previous_cell = cell;
          if subtract_u { previous_cell.u -= 1; }
          if subtract_v { previous_cell.v -= 1; }
          if subtract_w { previous_cell.w -= 1; }
          previous_cell
        };
        let shared_indices_index = Self::shared_index(previous_cell, vertex_data.vertex_index());
        debug_assert!(shared_indices.contains(shared_indices_index), "Tried to read out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell);
        let index = shared_indices[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_mesh);
        index
      }
    }
  }

  #[inline]
  pub fn create_vertex(
    vertex_data: &RegularVertexData,
    global_voxels: &[UVec3; 8],
    values: &[f32; 8],
    chunk_mesh: &mut ChunkMesh,
  ) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    debug_assert!(voxel_b_index > voxel_a_index, "Voxel B index {} is lower than voxel A index {}, which leads to inconsistencies", voxel_b_index, voxel_a_index);
    let pos_low = global_voxels[voxel_a_index as usize];
    let value_low = values[voxel_a_index as usize];
    let pos_high = global_voxels[voxel_b_index as usize];
    let value_high = values[voxel_b_index as usize];
    let position = Self::vertex_position(pos_low, value_low, pos_high, value_high);
    chunk_mesh.push_vertex(Vertex::new(position))
  }

  #[inline]
  pub fn vertex_position(pos_low: UVec3, value_low: f32, pos_high: UVec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    let pos_low = Vec3::from(pos_low);
    let pos_high = Vec3::from(pos_high);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline]
  pub fn shared_index(cell: RegularCell, vertex_index: u8) -> u32 {
    cell.u
      + C::CELLS_IN_CHUNK_ROW * cell.v
      + C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW * cell.w
      + C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW * C::CELLS_IN_CHUNK_ROW * vertex_index as u32
  }
}

/// Cell, local to the current chunk, in coordinate-space of the Transvoxel dissertation.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct RegularCell {
  pub u: u32,
  pub v: u32,
  pub w: u32,
}

impl RegularCell {
  #[inline]
  pub const fn new(u: u32, v: u32, w: u32) -> Self {
    Self { u, v, w }
  }

  #[inline]
  pub fn to_local_coordinate(&self, voxel: RegularVoxel) -> UVec3 {
    // NOTE: swaps Z and Y axis, as we use a left handed y-up coordinate system, whereas the dissertation uses a
    // left-handed z-up coordinate system.
    UVec3::new(self.u + voxel.u, self.w + voxel.w, self.v + voxel.v)
  }
}

/// Voxel, local to the current chunk, in coordinate-space of the Transvoxel dissertation.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct RegularVoxel {
  pub u: u32,
  pub v: u32,
  pub w: u32,
}

impl RegularVoxel {
  #[inline]
  pub const fn new(u: u32, v: u32, w: u32) -> Self {
    Self { u, v, w }
  }
}
