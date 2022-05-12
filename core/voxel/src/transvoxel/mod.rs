use std::marker::PhantomData;

use tracing::trace;
use ultraviolet::{UVec3, Vec3};

use gfx::prelude::*;

use crate::chunk::{ChunkMesh, ChunkSamples, ChunkSize, Vertex, Sliceable};
use crate::transvoxel::side::TransitionSide;
use crate::transvoxel::tables::TransitionVertexData;

pub mod side;
mod tables;

#[derive(Default, Copy, Clone)]
pub struct Transvoxel<C: ChunkSize> {
  _chunk_phantom: PhantomData<C>,
}

impl<C: ChunkSize> Transvoxel<C> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[profiling::function]
  pub fn extract_chunk(
    &self,
    side: TransitionSide,
    hires_chunk_mins: &[UVec3; 4],
    hires_chunk_samples: &[ChunkSamples<C>; 4],
    hires_step: u32,
    lores_min: UVec3,
    lores_step: u32,
    chunk_mesh: &mut ChunkMesh,
  ) {
    debug_assert!(C::CELLS_IN_CHUNK_ROW > 1, "Chunk size must be greater than one"); // OPTO: use compile-time assert.
    if side == TransitionSide::HiZ {
      trace!(
        "{:?} hires_chunk_mins: [0={: >4} 1={: >4} 2={: >4} 3={: >4}], hires_step: {: >4}, lores_min: {: >4}, lores_step: {: >4}",
        side,
        hires_chunk_mins[0].display(),
        hires_chunk_mins[1].display(),
        hires_chunk_mins[2].display(),
        hires_chunk_mins[3].display(),
        hires_step,
        lores_min.display(),
        lores_step
      );
    }
    let mut shared_indices = C::create_transvoxel_shared_indices_array(u16::MAX); // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
    for cell_v in 0..C::CELLS_IN_CHUNK_ROW {
      for cell_u in 0..C::CELLS_IN_CHUNK_ROW {
        Self::extract_cell(
          side,
          cell_u,
          cell_v,
          hires_chunk_mins,
          hires_chunk_samples,
          hires_step,
          lores_min,
          lores_step,
          &mut shared_indices,
          chunk_mesh,
        );
      }
    }
  }

  #[inline]
  fn extract_cell(
    side: TransitionSide,
    u: u32,
    v: u32,
    hires_chunk_mins: &[UVec3; 4],
    hires_chunk_samples: &[ChunkSamples<C>; 4],
    hires_step: u32,
    lores_min: UVec3,
    lores_step: u32,
    shared_indices: &mut C::TransvoxelSharedIndicesArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    // Get local voxels (i.e., the coordinates of all the 9 corners) of the high-resolution side of the transition cell.
    let hires_local_voxels = side.get_hires_local_voxels::<C>(u, v);
    let lores_local_voxels = side.get_lores_local_voxels::<C>(u, v);
    // Get which ChunkSamples we have to sample values from, and what their minimum is in their coordinate system.
    let (hires_min, hires_chunk_samples) = {
      let idx = (u / C::HALF_CELLS_IN_CHUNK_ROW) + (2 * (v / C::HALF_CELLS_IN_CHUNK_ROW)); // 0 = 0,0 | 1 = 1,0 | 2 = 0,1 | 3 = 1,1
      let idx = idx as usize;
      (hires_chunk_mins[idx], &hires_chunk_samples[idx])
    };
    // Get the global voxels of the cell.
    // OPTO: don't create global voxels here, as it may not be needed later, so we might be doing unnecessary work.
    let global_voxels: [Vec3; 13] = {
      let hires_local_voxels: [Vec3; 9] = [
        hires_local_voxels[0].into(),
        hires_local_voxels[1].into(),
        hires_local_voxels[2].into(),
        hires_local_voxels[3].into(),
        hires_local_voxels[4].into(),
        hires_local_voxels[5].into(),
        hires_local_voxels[6].into(),
        hires_local_voxels[7].into(),
        hires_local_voxels[8].into(),
      ];
      let hires_min: Vec3 = hires_min.into();
      let hires_step = hires_step as f32;
      let lores_min: Vec3 = lores_min.into();
      let lores_step = lores_step as f32;
      [
        hires_min + hires_step * hires_local_voxels[0], // 0
        hires_min + hires_step * hires_local_voxels[1], // 1
        hires_min + hires_step * hires_local_voxels[2], // 2
        hires_min + hires_step * hires_local_voxels[3], // 3
        hires_min + hires_step * hires_local_voxels[4], // 4
        hires_min + hires_step * hires_local_voxels[5], // 5
        hires_min + hires_step * hires_local_voxels[6], // 6
        hires_min + hires_step * hires_local_voxels[7], // 7
        hires_min + hires_step * hires_local_voxels[8], // 8
        lores_min + lores_step * lores_local_voxels[0], // 9
        lores_min + lores_step * lores_local_voxels[1], // A
        lores_min + lores_step * lores_local_voxels[2], // B
        lores_min + lores_step * lores_local_voxels[3], // C
      ]
    };
    // Sample the volume at each local voxel, producing values.
    let values = { // OPTO: can we make the rest of this code more efficient if an entire chunk is zero/positive/negative?
      let value_0_and_9 = hires_chunk_samples.sample(hires_local_voxels[0]);
      let value_2_and_a = hires_chunk_samples.sample(hires_local_voxels[2]);
      let value_6_and_b = hires_chunk_samples.sample(hires_local_voxels[6]);
      let value_8_and_c = hires_chunk_samples.sample(hires_local_voxels[8]);
      [
        value_0_and_9,
        hires_chunk_samples.sample(hires_local_voxels[1]),
        value_2_and_a,
        hires_chunk_samples.sample(hires_local_voxels[3]),
        hires_chunk_samples.sample(hires_local_voxels[4]),
        hires_chunk_samples.sample(hires_local_voxels[5]),
        value_6_and_b,
        hires_chunk_samples.sample(hires_local_voxels[7]),
        value_8_and_c,
        value_0_and_9,
        value_2_and_a,
        value_6_and_b,
        value_8_and_c,
      ]
    };

    // Create the case number by summing the contributions.
    let case = {
      let mut case = 0x0;
      if values[0].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[0] }
      if values[1].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[1] }
      if values[2].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[2] }
      if values[3].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[3] }
      if values[4].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[4] }
      if values[5].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[5] }
      if values[6].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[6] }
      if values[7].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[7] }
      if values[8].is_sign_positive() { case += tables::TRANSITION_VOXEL_CASE_CONTRIBUTION[8] }
      case
    };

    if side == TransitionSide::HiZ {
      trace!(
        "u:{: >2} v:{: >2} | HR[0={: >4} 1={: >4} 2={: >4} 3={: >4} 4={: >4} 5={: >4} 6={: >4} 7={: >4} 8={: >4}] LR[9={: >4} A={: >4} B={: >4} C={: >4}]",
        u,
        v,
        hires_local_voxels[0].display(),
        hires_local_voxels[1].display(),
        hires_local_voxels[2].display(),
        hires_local_voxels[3].display(),
        hires_local_voxels[4].display(),
        hires_local_voxels[5].display(),
        hires_local_voxels[6].display(),
        hires_local_voxels[7].display(),
        hires_local_voxels[8].display(),
        lores_local_voxels[0].display(),
        lores_local_voxels[1].display(),
        lores_local_voxels[2].display(),
        lores_local_voxels[3].display(),
      );
      trace!(
        "{case: <3}       | GV[0={: >4} 1={: >4} 2={: >4} 3={: >4} 4={: >4} 5={: >4} 6={: >4} 7={: >4} 8={: >4}     9={: >4} A={: >4} B={: >4} C={: >4}]",
        global_voxels[0].display(),
        global_voxels[1].display(),
        global_voxels[2].display(),
        global_voxels[3].display(),
        global_voxels[4].display(),
        global_voxels[5].display(),
        global_voxels[6].display(),
        global_voxels[7].display(),
        global_voxels[8].display(),
        global_voxels[9].display(),
        global_voxels[10].display(),
        global_voxels[11].display(),
        global_voxels[12].display(),
      );
    }

    if case == 0 || case == 511 { // No triangles // OPTO: use bit twiddling to break it down to 1 comparison?
      return;
    }

    // Get the cell class for the `case`.
    let raw_cell_class = tables::TRANSITION_CELL_CLASS[case as usize];
    let cell_class = raw_cell_class & 0x7F;
    // High bit of the class index indicates that the triangulation is inverted.
    let invert_triangulation = (raw_cell_class & 0x80) != 0;
    let invert_triangulation = !invert_triangulation; // We use LowZ as base case so everything is inverted?
    // Get the triangulation info corresponding to the cell class. This uses `cell_class` instead of `case`, because the
    // triangulation info is equivalent for a class of cells. The full `case` is used along with this info to form the
    // eventual triangles.
    let triangulation_info = tables::TRANSITION_CELL_DATA[cell_class as usize];
    let vertex_count = triangulation_info.get_vertex_count() as usize;
    let triangle_count = triangulation_info.get_triangle_count();
    // Get the vertex data corresponding to the `case`.
    let vertices_data = tables::TRANSITION_VERTEX_DATA[case as usize];

    // Get indices for all vertices, creating new vertices and thus new indices, or reusing indices from previous cells.
    let mut cell_vertices_indices: [u16; 12] = [0; 12];
    for (i, vd) in vertices_data.iter().enumerate() {
      if i >= vertex_count {
        break;
      }
      cell_vertices_indices[i] = Self::create_or_reuse_vertex(TransitionVertexData(*vd), u, v, &global_voxels, &values, shared_indices, chunk_mesh);
    }

    // Write the indices that form the triangulation of this transition cell.
    for t in 0..triangle_count {
      let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
      let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
      let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
      let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
      let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
      let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
      if invert_triangulation {
        chunk_mesh.push_index(global_index_1);
        chunk_mesh.push_index(global_index_2);
        chunk_mesh.push_index(global_index_3);
      } else {
        chunk_mesh.push_index(global_index_3);
        chunk_mesh.push_index(global_index_2);
        chunk_mesh.push_index(global_index_1);
      }
    }
  }

  #[inline]
  fn create_or_reuse_vertex(
    vertex_data: TransitionVertexData,
    u: u32,
    v: u32,
    global_voxels: &[Vec3; 13],
    values: &[f32; 13],
    shared_indices: &mut C::TransvoxelSharedIndicesArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) -> u16 {
    if vertex_data.new_reusable_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_mesh);
      let shared_indices_index = Self::shared_index(u, v, vertex_data.vertex_index() as usize);
      debug_assert!(shared_indices_index < shared_indices.slice().len(), "Tried to write out of bounds shared transition index, at index: {}, position: {}, {}", shared_indices_index, u, v);
      debug_assert!(shared_indices.slice()[shared_indices_index] == u16::MAX, "Tried to write already set shared transition index, at index: {}, position: {}, {}", shared_indices_index, u, v);
      shared_indices.slice_mut()[shared_indices_index] = index;
      index
    } else if vertex_data.new_interior_vertex() {
      // Create a new vertex and index, but this vertex will never be shared, as it is an interior vertex.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_mesh);
      index
    } else {
      let subtract_u = vertex_data.subtract_u();
      let subtract_v = vertex_data.subtract_v();
      // OPTO: use 3-bit validity mask as proposed in the paper?
      if (u > 0 || !subtract_u) && (v > 0 || !subtract_v) {
        // Return index of previous vertex.
        let (prev_u, prev_v) = {
          let mut prev_u = u;
          let mut prev_v = v;
          if subtract_u { prev_u -= 1; }
          if subtract_v { prev_v -= 1; }
          (prev_u, prev_v)
        };
        let shared_indices_index = Self::shared_index(prev_u, prev_v, vertex_data.vertex_index() as usize);
        debug_assert!(shared_indices_index < shared_indices.slice().len(), "Tried to read out of bounds shared transition index, at index: {}, position: {}, {}", shared_indices_index, prev_u, prev_v);
        let index = shared_indices.slice()[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared transition index, at index: {}, position: {}, {}", shared_indices_index, prev_u, prev_v);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, global_voxels, values, chunk_mesh);
        index
      }
    }
  }

  #[inline]
  fn create_vertex(
    vertex_data: TransitionVertexData,
    global_voxels: &[Vec3; 13],
    values: &[f32; 13],
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
  fn vertex_position(pos_low: Vec3, value_low: f32, pos_high: Vec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline]
  fn shared_index(u: u32, v: u32, vertex_index: usize) -> usize {
    u as usize
      + C::CELLS_IN_CHUNK_ROW_USIZE * v as usize
      + C::CELLS_IN_CHUNK_ROW_USIZE * C::CELLS_IN_CHUNK_ROW_USIZE * vertex_index
  }
}
