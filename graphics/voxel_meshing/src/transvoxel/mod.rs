use ultraviolet::{UVec2, UVec3, Vec3};

use crate::chunk::{CELLS_IN_CHUNK_ROW, CELLS_IN_CHUNK_ROW_USIZE, Chunk, ChunkSamples, Vertex};
use crate::transvoxel::side::TransitionSide;
use crate::transvoxel::tables::TransitionVertexData;

pub mod side;
mod tables;

#[derive(Copy, Clone)]
pub struct Transvoxel;

impl Transvoxel {
  const SHARED_INDICES_SIZE: usize = 10 * CELLS_IN_CHUNK_ROW_USIZE * CELLS_IN_CHUNK_ROW_USIZE;

  pub fn extract_chunk(
    &self,
    side: TransitionSide,
    hires_chunk_mins: &[UVec3; 4],
    hires_chunk_samples: &[ChunkSamples; 4],
    hires_step: u32,
    lores_min: UVec3,
    lores_step: u32,
    chunk: &mut Chunk,
  ) {
    let mut shared_indices = [u16::MAX; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
    for cell_y in 0..CELLS_IN_CHUNK_ROW {
      for cell_x in 0..CELLS_IN_CHUNK_ROW {
        let cell = UVec2::new(cell_x, cell_y);
        Self::extract_cell(
          side,
          cell,
          hires_chunk_mins,
          hires_chunk_samples,
          hires_step,
          lores_min,
          lores_step,
          &mut shared_indices,
          chunk,
        );
      }
    }
  }

  #[inline]
  fn extract_cell(
    side: TransitionSide,
    cell: UVec2,
    hires_chunk_mins: &[UVec3; 4],
    hires_chunk_samples: &[ChunkSamples; 4],
    hires_step: u32,
    lores_min: UVec3,
    lores_step: u32,
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) {
    // Get local voxels (i.e., the coordinates of all the 9 corners) of the high-resolution side of the transition cell.
    let hires_local_voxels = side.get_hires_local_voxels(cell);
    let lores_local_voxels = side.get_lores_local_voxels(cell);
    // Get which ChunkSamples we have to sample values from, and what their minimum is in their coordinate system.
    let (hires_min, hires_chunk_samples) = {
      let idx = (cell.x / 8) + (2 * (cell.y / 8)); // 0 = 0,0 | 1 = 1,0 | 2 = 0,1 | 3 = 1,1
      let idx = idx as usize;
      (hires_chunk_mins[idx], &hires_chunk_samples[idx])
    };
    // Get the global voxels of the cell.
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
    if case == 0 || case == 511 { // No triangles // OPTO: use bit twiddling to break it down to 1 comparison?
      return;
    }

    // Get the cell class for the `case`.
    let raw_cell_class = tables::TRANSITION_CELL_CLASS[case as usize];
    let cell_class = raw_cell_class & 0x7F;
    // High bit of the class index indicates that the triangulation is inverted.
    let invert_triangulation = (raw_cell_class & 0x80) != 0;
    let our_invert_triangulation = !invert_triangulation; // We use LowZ as base case so everything is inverted?
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
      let vertex_data = TransitionVertexData(*vd);
      cell_vertices_indices[i] = Self::create_or_reuse_vertex(vertex_data, cell, &global_voxels, &values, shared_indices, chunk);
    }

    // Write the indices that form the triangulation of this transition cell.
    for t in 0..triangle_count {
      let v1_index_in_cell = triangulation_info.vertex_index[3 * t as usize];
      let v2_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 1];
      let v3_index_in_cell = triangulation_info.vertex_index[3 * t as usize + 2];
      let global_index_1 = cell_vertices_indices[v1_index_in_cell as usize];
      let global_index_2 = cell_vertices_indices[v2_index_in_cell as usize];
      let global_index_3 = cell_vertices_indices[v3_index_in_cell as usize];
      if our_invert_triangulation {
        chunk.indices.push(global_index_1);
        chunk.indices.push(global_index_2);
        chunk.indices.push(global_index_3);
      } else {
        chunk.indices.push(global_index_3);
        chunk.indices.push(global_index_2);
        chunk.indices.push(global_index_1);
      }
    }
  }

  #[inline]
  fn create_or_reuse_vertex(
    vertex_data: TransitionVertexData,
    cell: UVec2,
    global_voxels: &[Vec3; 13],
    values: &[f32; 13],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) -> u16 {
    if vertex_data.new_reusable_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
      let shared_indices_index = Self::shared_index(cell, vertex_data.vertex_index() as usize);
      debug_assert!(shared_indices_index < shared_indices.len(), "Tried to write out of bounds shared transition index, at index: {}, position: {:?}", shared_indices_index, cell);
      debug_assert!(shared_indices[shared_indices_index] == u16::MAX, "Tried to write already set shared transition index, at index: {}, position: {:?}", shared_indices_index, cell);
      shared_indices[shared_indices_index] = index;
      index
    } else if vertex_data.new_interior_vertex() {
      // Create a new vertex and index, but this vertex will never be shared, as it is an interior vertex.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
      index
    } else {
      let subtract_x = vertex_data.subtract_x();
      let subtract_y = vertex_data.subtract_y();
      // OPTO: use 3-bit validity mask as proposed in the paper?
      if (cell.x > 0 || !subtract_x) && (cell.y > 0 || !subtract_y) {
        // Return index of previous vertex.
        let previous_cell_local = {
          let mut previous_cell_local = cell;
          if subtract_x { previous_cell_local.x -= 1; }
          if subtract_y { previous_cell_local.y -= 1; }
          previous_cell_local
        };
        let shared_indices_index = Self::shared_index(previous_cell_local, vertex_data.vertex_index() as usize);
        debug_assert!(shared_indices_index < shared_indices.len(), "Tried to read out of bounds shared transition index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        let index = shared_indices[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared transition index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
        index
      }
    }
  }

  #[inline]
  fn create_vertex(vertex_data: TransitionVertexData, global_voxels: &[Vec3; 13], values: &[f32; 13], chunk: &mut Chunk) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    debug_assert!(voxel_b_index > voxel_a_index, "Voxel B index {} is higher than voxel A index {}, which leads to inconsistencies", voxel_b_index, voxel_a_index);
    let position = Self::vertex_position(
      global_voxels[voxel_a_index as usize],
      values[voxel_a_index as usize],
      global_voxels[voxel_b_index as usize],
      values[voxel_b_index as usize],
    );
    let index = chunk.vertices.len() as u16;
    chunk.vertices.push(Vertex::new(position));
    index
  }

  #[inline]
  fn vertex_position(pos_low: Vec3, value_low: f32, pos_high: Vec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline]
  fn shared_index(cell: UVec2, vertex_index: usize) -> usize {
    cell.x as usize
      + CELLS_IN_CHUNK_ROW_USIZE * cell.y as usize
      + CELLS_IN_CHUNK_ROW_USIZE * CELLS_IN_CHUNK_ROW_USIZE * vertex_index
  }
}
