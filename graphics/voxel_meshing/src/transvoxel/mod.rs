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
    side: TransitionSide,
    lores_step: u32,
    hires_step: u32,
    hires_chunk_samples: &[ChunkSamples; 4],
    chunk: &mut Chunk,
  ) {
    let mut shared_indices = [u16::MAX; Self::SHARED_INDICES_SIZE]; // OPTO: reduce size and management of this array to the number of shared indices that we need to keep in memory?
    for cell_y in 0..CELLS_IN_CHUNK_ROW {
      for cell_x in 0..CELLS_IN_CHUNK_ROW {
        let cell = UVec2::new(cell_x, cell_y);
        Self::extract_cell(cell, start, side, lores_step, hires_step, hires_chunk_samples, &mut shared_indices, chunk);
      }
    }
  }

  #[inline]
  fn extract_cell(
    cell: UVec2,
    start: UVec3,
    side: TransitionSide,
    lores_step: u32,
    hires_step: u32,
    hires_chunk_samples: &[ChunkSamples; 4],
    shared_indices: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk: &mut Chunk,
  ) {
    // Get local voxels (i.e., the coordinates of all the 9 corners) of the high-resolution side of the transition cell.
    let hires_local_voxels = side.get_hires_local_voxels(cell);
    let lores_local_voxels = side.get_lores_local_voxels(cell);
    // Get the global voxels of the cell.
    let global_voxels: [Vec3; 13] = {
      let start: Vec3 = start.into();
      let hires_step = hires_step as f32;
      let lores_step = lores_step as f32;
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
      // TODO: `hires_local_voxels` are local to the high resolution chunks, whereas `start` is based on the low
      //       resolution chunk. Therefore, we cannot use `start` here to calculate the global coordinates of the high
      //       resolution voxels, but must use a separate start for the high resolution chunks!
      [
        start + hires_step * hires_local_voxels[0], // 0
        start + hires_step * hires_local_voxels[1], // 1
        start + hires_step * hires_local_voxels[2], // 2
        start + hires_step * hires_local_voxels[3], // 3
        start + hires_step * hires_local_voxels[4], // 4
        start + hires_step * hires_local_voxels[5], // 5
        start + hires_step * hires_local_voxels[6], // 6
        start + hires_step * hires_local_voxels[7], // 7
        start + hires_step * hires_local_voxels[8], // 8
        start + lores_step * lores_local_voxels[0], // 9
        start + lores_step * lores_local_voxels[1], // A
        start + lores_step * lores_local_voxels[2], // B
        start + lores_step * lores_local_voxels[3], // C
      ]
    };
    // Get which ChunkSamples we have to sample values from.
    let chunk_samples = {
      let chunk_samples_index = (cell.x / 8) + (2 * (cell.y / 8));
      &hires_chunk_samples[chunk_samples_index as usize]
    };
    // Sample the volume at each local voxel, producing values.
    let values = { // OPTO: can we make the rest of this code more efficient if an entire chunk is zero/positive/negative?
      let value_0_and_9 = chunk_samples.sample(hires_local_voxels[0]);
      let value_2_and_a = chunk_samples.sample(hires_local_voxels[2]);
      let value_6_and_b = chunk_samples.sample(hires_local_voxels[6]);
      let value_8_and_c = chunk_samples.sample(hires_local_voxels[8]);
      [
        value_0_and_9,
        chunk_samples.sample(hires_local_voxels[1]),
        value_2_and_a,
        chunk_samples.sample(hires_local_voxels[3]),
        chunk_samples.sample(hires_local_voxels[4]),
        chunk_samples.sample(hires_local_voxels[5]),
        value_6_and_b,
        chunk_samples.sample(hires_local_voxels[7]),
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
  pub fn get_hires_local_voxels(&self, cell: UVec2) -> [UVec3; 9] {
    match self {
      TransitionSide::LowX => {
        let cell_3d = UVec3::new(CELLS_IN_CHUNK_ROW, (cell.x % 8) * 2, (cell.y % 8) * 2);
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
        let cell_3d = UVec3::new((cell.x % 8) * 2, (cell.y % 8) * 2, CELLS_IN_CHUNK_ROW);
        [
          cell_3d + UVec3::new(0, 0, 0), // 0 & 9
          cell_3d + UVec3::new(1, 0, 0), // 1
          cell_3d + UVec3::new(2, 0, 0), // 2 & A
          cell_3d + UVec3::new(0, 1, 0), // 3
          cell_3d + UVec3::new(1, 1, 0), // 4
          cell_3d + UVec3::new(2, 1, 0), // 5
          cell_3d + UVec3::new(0, 2, 0), // 6 & B
          cell_3d + UVec3::new(1, 2, 0), // 7
          cell_3d + UVec3::new(2, 2, 0), // 8 & C
        ]
      }
      TransitionSide::HighZ => {
        todo!()
      }
    }
  }

  #[inline]
  pub fn get_lores_local_voxels(&self, cell: UVec2) -> [Vec3; 4] {
    match self {
      TransitionSide::LowX => {
        todo!()
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
        let cell_3d = Vec3::new(cell.x as f32, cell.y as f32, 0.5); // TODO: determine width of transition cell consistently.
        [
          cell_3d + Vec3::new(0.0, 0.0, 0.0), // 9
          cell_3d + Vec3::new(1.0, 0.0, 0.0), // A
          cell_3d + Vec3::new(0.0, 1.0, 0.0), // B
          cell_3d + Vec3::new(1.0, 1.0, 0.0), // C
        ]
      }
      TransitionSide::HighZ => {
        todo!()
      }
    }
  }
}
