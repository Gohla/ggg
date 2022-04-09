///! Surface nets implementation based on:
///!
///! * https://bonsairobo.medium.com/smooth-voxel-mapping-a-technical-deep-dive-on-real-time-surface-nets-and-texturing-ef06d0f8ca14
///! * https://github.com/bonsairobo/fast-surface-nets-rs

use std::marker::PhantomData;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::{cell_index_from_xyz, CellIndex, ChunkIndices, ChunkSampleArray, ChunkSamples, ChunkSize, ChunkVertices, Vertex, voxel_index_from_xyz, VoxelIndex};

#[derive(Default, Copy, Clone)]
pub struct SurfaceNets<C: ChunkSize> {
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize> SurfaceNets<C> {
  pub const SHARED_INDICES_SIZE: usize = C::CELLS_IN_CHUNK_USIZE;

  #[inline]
  pub fn new() -> Self { Self::default() }


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
      let mut cell_index_to_vertex_index = [u16::MAX; Self::SHARED_INDICES_SIZE];
      Self::extract_global_positions(min, step, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_vertices);
      Self::extract_quads(chunk_sample_array, &cell_index_to_vertex_index, chunk_vertices);
    }
  }

  fn extract_global_positions(
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut [u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        for x in 0..C::CELLS_IN_CHUNK_ROW {
          let cell = Cell::new(x, y, z);
          let cell_index = cell.to_index::<C>().into_usize();
          if let Some(position) = Self::extract_cell_vertex_positions(cell, min, step, chunk_sample_array) {
            let vertex_index = chunk_vertices.push_vertex(Vertex { position });
            debug_assert!(cell_index < cell_index_to_vertex_index.len(), "Tried to write out of bounds cell index {} in cell index to vertex index array, with vertex index: {}", cell_index, vertex_index);
            debug_assert!(cell_index_to_vertex_index[cell_index] == u16::MAX, "Tried to write to already written cell index {} in cell index to vertex index array, with vertex index: {}", cell_index, vertex_index);
            debug_assert!(vertex_index < u16::MAX, "Tried to write vertex index {} that is equal to or larger than {} in cell index to vertex index array, at cell index: {}", vertex_index, u16::MAX, cell_index);
            cell_index_to_vertex_index[cell_index] = vertex_index as u16;
          }
        }
      }
    }
  }

  #[inline]
  fn extract_cell_vertex_positions(
    cell: Cell,
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
  ) -> Option<Vec3> where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    let local_voxel_positions = Self::local_voxel_positions(cell);
    let values = Self::sample(chunk_sample_array, &local_voxel_positions);
    if !Self::has_point(&values) { return None; }
    let global_voxel_positions = Self::global_voxel_positions(min, step, &local_voxel_positions);
    let vertex_position = Self::centroid_of_edge_intersections(&values, &global_voxel_positions);
    Some(vertex_position)
  }

  pub const VOXELS: [Voxel; 8] = [
    Voxel::new(0, 0, 0), // 1
    Voxel::new(1, 0, 0), // 2
    Voxel::new(0, 1, 0), // 3
    Voxel::new(1, 1, 0), // 4
    Voxel::new(0, 0, 1), // 5
    Voxel::new(1, 0, 1), // 6
    Voxel::new(0, 1, 1), // 7
    Voxel::new(1, 1, 1), // 8
  ];

  #[inline]
  pub fn local_voxel_positions(cell: Cell) -> [UVec3; 8] {
    [
      cell.to_local_position(Self::VOXELS[0]),
      cell.to_local_position(Self::VOXELS[1]),
      cell.to_local_position(Self::VOXELS[2]),
      cell.to_local_position(Self::VOXELS[3]),
      cell.to_local_position(Self::VOXELS[4]),
      cell.to_local_position(Self::VOXELS[5]),
      cell.to_local_position(Self::VOXELS[6]),
      cell.to_local_position(Self::VOXELS[7]),
    ]
  }

  #[inline]
  pub fn sample(chunk_sample_array: &ChunkSampleArray<C>, local_coordinates: &[UVec3; 8]) -> [f32; 8] where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
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
  pub fn has_point(values: &[f32; 8]) -> bool {
    let sign_bits = (values[0].is_sign_negative() as u8) << 0
      | (values[1].is_sign_negative() as u8) << 1
      | (values[2].is_sign_negative() as u8) << 2
      | (values[3].is_sign_negative() as u8) << 3
      | (values[4].is_sign_negative() as u8) << 4
      | (values[5].is_sign_negative() as u8) << 5
      | (values[6].is_sign_negative() as u8) << 6
      | (values[7].is_sign_negative() as u8) << 7;
    sign_bits != 0 && sign_bits != 255 // OPTO: use bit twiddling to break it down to 1 comparison?
  }

  #[inline]
  pub fn global_voxel_positions(min: UVec3, step: u32, local_voxel_positions: &[UVec3; 8]) -> [UVec3; 8] {
    [
      min + step * local_voxel_positions[0],
      min + step * local_voxel_positions[1],
      min + step * local_voxel_positions[2],
      min + step * local_voxel_positions[3],
      min + step * local_voxel_positions[4],
      min + step * local_voxel_positions[5],
      min + step * local_voxel_positions[6],
      min + step * local_voxel_positions[7],
    ]
  }

  pub const EDGE_TO_VOXEL_INDICES: [[u8; 2]; 12] = [ // OPTO: compact to single u8 value per edge pair.
    [0b000, 0b001],
    [0b000, 0b010],
    [0b000, 0b100],
    [0b001, 0b011],
    [0b001, 0b101],
    [0b010, 0b011],
    [0b010, 0b110],
    [0b011, 0b111],
    [0b100, 0b101],
    [0b100, 0b110],
    [0b101, 0b111],
    [0b110, 0b111],
  ];

  #[inline]
  pub fn centroid_of_edge_intersections(
    values: &[f32; 8],
    global_voxel_positions: &[UVec3; 8],
  ) -> Vec3 {
    let mut count = 0;
    let mut sum = Vec3::zero();
    for &[voxel_a_index, voxel_b_index] in Self::EDGE_TO_VOXEL_INDICES.iter() {
      let voxel_a_index = voxel_a_index as usize;
      let value_a = values[voxel_a_index];
      let voxel_b_index = voxel_b_index as usize;
      let value_b = values[voxel_b_index];
      if (value_a < 0.0) != (value_b < 0.0) {
        count += 1;
        let position_a = global_voxel_positions[voxel_a_index];
        let position_b = global_voxel_positions[voxel_b_index];
        sum += Self::surface_edge_intersection(position_a, value_a, position_b, value_b);
      }
    }
    sum / count as f32
  }

  #[inline]
  fn surface_edge_intersection(position_a: UVec3, value_a: f32, position_b: UVec3, value_b: f32) -> Vec3 {
    let t = value_a / (value_a - value_b);
    let position_a = Vec3::from(position_a);
    let position_b = Vec3::from(position_b);
    t * position_b + (1.0 - t) * position_a
  }


  fn extract_quads(
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &[u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        for x in 0..C::CELLS_IN_CHUNK_ROW {
          let cell = Cell::new(x, y, z);
          let cell_index = cell.to_index::<C>();
          let min_voxel_index = cell.to_min_voxel_index::<C>();
          let vertex_index = cell_index_to_vertex_index[cell_index.into_usize()];
          if vertex_index == u16::MAX { continue; }
          Self::extract_quad(cell, cell_index, min_voxel_index, chunk_sample_array, cell_index_to_vertex_index, chunk_vertices);
        }
      }
    }
  }

  fn extract_quad(
    cell: Cell,
    cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &[u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    // Do edges parallel with the X axis
    if cell.y != 0 && cell.z != 0 && cell.x < C::CELLS_IN_CHUNK_ROW {
      Self::maybe_make_quad(
        chunk_sample_array,
        cell_index_to_vertex_index,
        chunk_vertices,
        min_voxel_index,
        min_voxel_index + Self::ADD_X_VOXEL_INDEX_OFFSET,
        cell_index,
        Self::ADD_Y_CELL_INDEX_OFFSET,
        Self::ADD_Z_CELL_INDEX_OFFSET,
      );
    }
    // Do edges parallel with the Y axis
    if cell.x != 0 && cell.z != 0 && cell.y < C::CELLS_IN_CHUNK_ROW {
      Self::maybe_make_quad(
        chunk_sample_array,
        cell_index_to_vertex_index,
        chunk_vertices,
        min_voxel_index,
        min_voxel_index + Self::ADD_Y_VOXEL_INDEX_OFFSET,
        cell_index,
        Self::ADD_Z_CELL_INDEX_OFFSET,
        Self::ADD_X_CELL_INDEX_OFFSET,
      );
    }
    // Do edges parallel with the Z axis
    if cell.x != 0 && cell.y != 0 && cell.z < C::CELLS_IN_CHUNK_ROW {
      Self::maybe_make_quad(
        chunk_sample_array,
        cell_index_to_vertex_index,
        chunk_vertices,
        min_voxel_index,
        min_voxel_index + Self::ADD_Z_VOXEL_INDEX_OFFSET,
        cell_index,
        Self::ADD_X_CELL_INDEX_OFFSET,
        Self::ADD_Y_CELL_INDEX_OFFSET,
      );
    }
  }

  const ADD_X_VOXEL_INDEX_OFFSET: VoxelIndex = voxel_index_from_xyz::<C>(1, 0, 0);
  const ADD_Y_VOXEL_INDEX_OFFSET: VoxelIndex = voxel_index_from_xyz::<C>(0, 1, 0);
  const ADD_Z_VOXEL_INDEX_OFFSET: VoxelIndex = voxel_index_from_xyz::<C>(0, 0, 1);
  const ADD_X_CELL_INDEX_OFFSET: CellIndex = cell_index_from_xyz::<C>(1, 0, 0);
  const ADD_Y_CELL_INDEX_OFFSET: CellIndex = cell_index_from_xyz::<C>(0, 1, 0);
  const ADD_Z_CELL_INDEX_OFFSET: CellIndex = cell_index_from_xyz::<C>(0, 0, 1);

  fn maybe_make_quad(
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &[u16; Self::SHARED_INDICES_SIZE],
    chunk_vertices: &mut ChunkVertices,
    voxel_index_a: VoxelIndex,
    voxel_index_b: VoxelIndex,
    cell_index: CellIndex,
    axis_b_cell_index_offset: CellIndex,
    axis_c_cell_index_offset: CellIndex,
  ) where
    [f32; C::VOXELS_IN_CHUNK_USIZE]:
  {
    let value_a = chunk_sample_array.sample_index(voxel_index_a);
    let value_b = chunk_sample_array.sample_index(voxel_index_b);
    let negative_face = match (value_a.is_sign_negative(), value_b.is_sign_negative()) {
      (true, false) => false,
      (false, true) => true,
      _ => return, // No face.
    };

    // The triangle points, viewed face-front, look like this:
    // v1 v3
    // v2 v4
    let v1 = cell_index_to_vertex_index[cell_index.into_usize()];
    let v2 = cell_index_to_vertex_index[(cell_index - axis_b_cell_index_offset).into_usize()];
    let v3 = cell_index_to_vertex_index[(cell_index - axis_c_cell_index_offset).into_usize()];
    let v4 = cell_index_to_vertex_index[(cell_index - axis_b_cell_index_offset - axis_c_cell_index_offset).into_usize()];
    let (pos1, pos2, pos3, pos4) = (
      chunk_vertices.vertices()[v1 as usize].position,
      chunk_vertices.vertices()[v2 as usize].position,
      chunk_vertices.vertices()[v3 as usize].position,
      chunk_vertices.vertices()[v4 as usize].position,
    );
    // Split the quad along the shorter axis, rather than the longer one.
    let distance_a = (pos4 - pos1).mag_sq(); // pos1.distance_squared(pos4)
    let distance_b = (pos3 - pos2).mag_sq(); // pos2.distance_squared(pos3)
    let quad = if distance_a < distance_b {
      if negative_face {
        [v1, v4, v2, v1, v3, v4]
      } else {
        [v1, v2, v4, v1, v4, v3]
      }
    } else if negative_face {
      [v2, v3, v4, v2, v1, v3]
    } else {
      [v2, v4, v3, v2, v3, v1]
    };
    chunk_vertices.extend_indices_from_slice(&quad);
  }
}

/// Position of the minimal corner (left, bottom, back) of a cell, local to the current chunk.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Cell {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}

impl Cell {
  #[inline]
  pub const fn new(x: u32, y: u32, z: u32) -> Self {
    Self { x, y, z }
  }

  #[inline]
  pub fn to_local_position(&self, voxel: Voxel) -> UVec3 {
    UVec3::new(self.x + voxel.x, self.y + voxel.y, self.z + voxel.z)
  }

  #[inline]
  pub fn to_index<C: ChunkSize>(&self) -> CellIndex {
    C::cell_index_from_xyz(self.x, self.y, self.z)
  }

  #[inline]
  pub fn to_min_voxel_index<C: ChunkSize>(&self) -> VoxelIndex {
    C::voxel_index_from_xyz(self.x, self.y, self.z)
  }
}

/// Position of a voxel, local to the current chunk.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Voxel {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}

impl Voxel {
  #[inline]
  pub const fn new(x: u32, y: u32, z: u32) -> Self {
    Self { x, y, z }
  }
}

