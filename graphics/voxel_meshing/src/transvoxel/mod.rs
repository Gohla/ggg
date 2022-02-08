use ultraviolet::{UVec2, UVec3, Vec3};

use crate::chunk::{CELLS_IN_CHUNK_ROW, CELLS_IN_CHUNK_ROW_USIZE, Chunk, ChunkSamples, Vertex};
use crate::transvoxel::tables::TransitionVertexData;

mod tables;

#[derive(Copy, Clone)]
pub struct Transvoxel;

impl Transvoxel {
  const SHARED_INDICES_SIZE: usize = 10 * CELLS_IN_CHUNK_ROW_USIZE * CELLS_IN_CHUNK_ROW_USIZE;

  pub fn extract_chunk(
    &self,
    start: UVec3,
    high_resolution_step: u32,
    side: TransitionSide,
    chunk_samples: &[ChunkSamples; 4],
    chunk: &mut Chunk,
  ) {
    let mut shared_indices = [u16::MAX; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
    for cell_u in 0..CELLS_IN_CHUNK_ROW {
      for cell_v in 0..CELLS_IN_CHUNK_ROW {
        let cell = UVec2::new(cell_u, cell_v);
        Self::extract_cell(cell, side, start, high_resolution_step, chunk_samples, &mut shared_indices, chunk);
      }
    }
  }

  #[inline]
  fn extract_cell(
    cell: UVec2,
    side: TransitionSide,
    start: UVec3,
    high_resolution_step: u32,
    chunk_samples: &[ChunkSamples; 4],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) {
    // Get local voxels (i.e., the coordinates of all the 9 corners) of the transition cell.
    let local_voxels = side.get_transition_voxels(cell);
    // Get the global voxels of the cell.
    let global_voxels = [
      start + high_resolution_step * local_voxels[0],
      start + high_resolution_step * local_voxels[1],
      start + high_resolution_step * local_voxels[2],
      start + high_resolution_step * local_voxels[3],
      start + high_resolution_step * local_voxels[4],
      start + high_resolution_step * local_voxels[5],
      start + high_resolution_step * local_voxels[6],
      start + high_resolution_step * local_voxels[7],
      start + high_resolution_step * local_voxels[8],
    ];
    // Get which ChunkSamples we have to sample values from.
    let chunk_samples = {
      let chunk_samples_index = (cell.x / 8) + (2 * (cell.y / 8));
      &chunk_samples[chunk_samples_index as usize]
    };
    // Sample the volume at each local voxel, producing values.
    let values = [ // OPTO: can we make the rest of this code more efficient if an entire chunk is zero/positive/negative?
      chunk_samples.sample(local_voxels[0]),
      chunk_samples.sample(local_voxels[1]),
      chunk_samples.sample(local_voxels[2]),
      chunk_samples.sample(local_voxels[3]),
      chunk_samples.sample(local_voxels[4]),
      chunk_samples.sample(local_voxels[5]),
      chunk_samples.sample(local_voxels[6]),
      chunk_samples.sample(local_voxels[7]),
      chunk_samples.sample(local_voxels[8]),
    ];

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
    global_voxels: &[UVec3; 9],
    values: &[f32; 9],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) -> u16 {
    if vertex_data.new_reusable_vertex() {
      // Create a new vertex and index, and share the index.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
      let shared_indices_index = Self::shared_index(cell, vertex_data.vertex_index() as usize);
      debug_assert!(shared_indices_index < shared_indices.len(), "Tried to write out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      debug_assert!(shared_indices[shared_indices_index] == u16::MAX, "Tried to write already set shared index, at index: {}, position: {:?}", shared_indices_index, cell);
      shared_indices[shared_indices_index] = index;
      index
    } else if vertex_data.new_interior_vertex() {
      // Create a new vertex and index, but this vertex will never be shared, as it is an interior vertex.
      let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
      index
    } else {
      let subtract_u = vertex_data.subtract_u();
      let subtract_v = vertex_data.subtract_v();
      // OPTO: use 3-bit validity mask as proposed in the paper?
      if (cell.x > 0 || !subtract_u) && (cell.y > 0 || !subtract_v) {
        // Return index of previous vertex.
        let previous_cell_local = {
          let mut previous_cell_local = cell;
          if subtract_u { previous_cell_local.x -= 1; }
          if subtract_v { previous_cell_local.y -= 1; }
          previous_cell_local
        };
        let shared_indices_index = Self::shared_index(previous_cell_local, vertex_data.vertex_index() as usize);
        debug_assert!(shared_indices_index < shared_indices.len(), "Tried to read out of bounds shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        let index = shared_indices[shared_indices_index];
        debug_assert!(index != u16::MAX, "Tried to read unset shared index, at index: {}, position: {:?}", shared_indices_index, previous_cell_local);
        index
      } else {
        // Create a new vertex and index, but this vertex will never be shared, as it occurs on the minimal boundary.
        let index = Self::create_vertex(vertex_data, global_voxels, values, chunk);
        index
      }
    }
  }

  #[inline]
  fn create_vertex(vertex_data: TransitionVertexData, global_voxels: &[UVec3; 9], values: &[f32; 9], chunk: &mut Chunk) -> u16 {
    let voxel_a_index = vertex_data.voxel_a_index();
    let voxel_b_index = vertex_data.voxel_b_index();
    debug_assert!(voxel_b_index > voxel_a_index);
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
  fn vertex_position(pos_low: UVec3, value_low: f32, pos_high: UVec3, value_high: f32) -> Vec3 {
    let t = value_high / (value_high - value_low);
    let pos_low = Vec3::from(pos_low);
    let pos_high = Vec3::from(pos_high);
    t * pos_low + (1.0 - t) * pos_high
  }

  #[inline]
  fn shared_index(cell: UVec2, vertex_index: usize) -> usize {
    cell.x as usize
      + CELLS_IN_CHUNK_ROW_USIZE * cell.y as usize
      + CELLS_IN_CHUNK_ROW_USIZE * CELLS_IN_CHUNK_ROW_USIZE * vertex_index
  }
}

// Transition sides

flagset::flags! {
   pub enum TransitionSide: u8 {
        LowX,
        HighX,
        LowY,
        HighY,
        LowZ,
        HighZ,
    }
}
pub type TransitionSides = flagset::FlagSet<TransitionSide>;

impl TransitionSide {
  #[inline]
  pub fn get_transition_voxels(&self, cell: UVec2) -> [UVec3; 9] {
    match self {
      TransitionSide::LowX => {
        let cell_3d = UVec3::new(16, (cell.x % 8) * 2, (cell.y % 8) * 2);
        [
          cell_3d + UVec3::new(0, 0, 0), // 0 & 9
          cell_3d + UVec3::new(0, 0, 1), // 1
          cell_3d + UVec3::new(0, 0, 2), // 2 & A
          cell_3d + UVec3::new(0, 1, 0), // 3
          cell_3d + UVec3::new(0, 1, 1), // 4
          cell_3d + UVec3::new(0, 1, 2), // 5
          cell_3d + UVec3::new(0, 2, 0), // 6 & B
          cell_3d + UVec3::new(0, 2, 1), // 7
          cell_3d + UVec3::new(0, 2, 2), // 8 & C
        ]
      }
      TransitionSide::HighX => {
        todo!()
      }
      TransitionSide::LowY => {
        todo!()
      }
      TransitionSide::HighY => {
        todo!()
      }
      TransitionSide::LowZ => {
        todo!()
      }
      TransitionSide::HighZ => {
        todo!()
      }
    }
  }
}
