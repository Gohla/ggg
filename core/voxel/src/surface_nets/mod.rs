///! Surface nets implementation based on:
///!
///! * https://bonsairobo.medium.com/smooth-voxel-mapping-a-technical-deep-dive-on-real-time-surface-nets-and-texturing-ef06d0f8ca14
///! * https://github.com/bonsairobo/fast-surface-nets-rs

use std::marker::PhantomData;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::array::Array as ArrayTrait;
use crate::chunk::index::{CellIndex, VoxelIndex};
use crate::chunk::mesh::{ChunkMesh, Vertex};
use crate::chunk::sample::{ChunkSampleArray, ChunkSamples};
use crate::chunk::shape::Shape as ShapeTrait;
use crate::chunk::size::ChunkSize;

pub mod lod;

type CellShape<C> = <C as ChunkSize>::CellChunkShape;
type VoxelShape<C> = <C as ChunkSize>::VoxelChunkShape;
type Array<C> = <C as ChunkSize>::CellChunkArray<u16>;

#[derive(Default, Copy, Clone)]
pub struct SurfaceNets<C: ChunkSize> {
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize> SurfaceNets<C> {
  #[inline]
  pub fn new() -> Self { Self::default() }


  // Top-level functions
  #[profiling::function]
  pub fn extract_chunk(
    &self,
    min: UVec3,
    step: u32,
    chunk_samples: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      let mut cell_index_to_vertex_index = Array::<C>::new(u16::MAX);
      Self::extract_global_positions::<CellShape<C>>(min, step, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
      Self::extract_quads(chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
  }


  // Extract positions

  fn extract_global_positions<S: ShapeTrait<CellIndex>>(
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    S::for_all(|x, y, z, cell_index| {
      let cell = Cell::new(x, y, z);
      Self::extract_global_position(cell, cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
    });
  }

  #[inline]
  fn extract_global_position(
    cell: Cell,
    cell_index: CellIndex,
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut impl ArrayTrait<u16, CellIndex>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if let Some(position) = Self::extract_cell_vertex_positions(cell, min, step, chunk_sample_array) {
      let vertex_index = chunk_mesh.push_vertex(Vertex { position });
      debug_assert!(cell_index_to_vertex_index.contains(cell_index), "Tried to write out of bounds cell index {} (>= {}) in cell index to vertex index array, with vertex index: {}", cell_index, cell_index_to_vertex_index.len(), vertex_index);
      debug_assert!(cell_index_to_vertex_index.index(cell_index) == u16::MAX, "Tried to write to already written cell index {} in cell index to vertex index array, with vertex index: {}", cell_index, vertex_index);
      debug_assert!(vertex_index < u16::MAX, "Tried to write vertex index {} that is equal to or larger than {} in cell index to vertex index array, at cell index: {}", vertex_index, u16::MAX, cell_index);
      cell_index_to_vertex_index.set(cell_index, vertex_index as u16);
    }
  }

  // Consider the grid-aligned cube where `min` is the minimal corner. Find a point inside this cube that is 
  // approximately on the isosurface.
  //
  // This is done by estimating, for each cube edge, where the isosurface crosses the edge (if it does at all). Then the
  // estimated surface point is the average of these edge crossings.
  #[inline]
  fn extract_cell_vertex_positions(
    cell: Cell,
    min: UVec3,
    step: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
  ) -> Option<Vec3> {
    let local_voxel_positions = Self::local_voxel_positions(cell);
    let values = Self::sample(chunk_sample_array, &local_voxel_positions);
    let case = Self::case(&values);
    if case == 0 || case == 255 { return None; } // OPTO: use bit twiddling to break it down to 1 comparison?
    let global_voxel_positions = Self::global_voxel_positions(min, step, &local_voxel_positions);
    let vertex_position = Self::centroid_of_edge_intersections(case, &values, &global_voxel_positions);
    Some(vertex_position)
  }

  #[inline]
  pub fn centroid_of_edge_intersections(
    case: u8,
    values: &[f32; 8],
    global_voxel_positions: &[Vec3; 8],
  ) -> Vec3 {
    let mut count = 0;
    let mut sum = Vec3::zero();
    for corner in &Self::EDGE_TO_VOXEL_INDICES {
      let voxel_a_index = corner >> 4 /* High nibble */;
      let voxel_b_index = corner & 0b0000_1111; /* Low nibble */
      let a_negative = (case & (1 << voxel_a_index)) != 0;
      let b_negative = (case & (1 << voxel_b_index)) != 0;
      if a_negative != b_negative {
        let voxel_a_index = voxel_a_index as usize;
        let voxel_b_index = voxel_b_index as usize;
        let value_a = values[voxel_a_index];
        let value_b = values[voxel_b_index];
        let position_a = global_voxel_positions[voxel_a_index];
        let position_b = global_voxel_positions[voxel_b_index];
        count += 1;
        sum += Self::surface_edge_intersection(position_a, value_a, position_b, value_b);
      }
    }
    sum / count as f32
  }

  // Given two cube corners, find the point between them where the SDF is zero. (This might not exist).
  #[inline]
  fn surface_edge_intersection(position_a: Vec3, value_a: f32, position_b: Vec3, value_b: f32) -> Vec3 {
    let t = value_a / (value_a - value_b);
    t * position_b + (1.0 - t) * position_a
  }


  // Extract quads

  fn extract_quads(
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) { // PERF: using Shape::for_all here decreases performance
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        for x in 0..C::CELLS_IN_CHUNK_ROW {
          let cell = Cell::new(x, y, z);
          let cell_index = cell.to_index::<C>();
          let min_voxel_index = cell.to_min_voxel_index::<C>();
          let vertex_index = cell_index_to_vertex_index.index(cell_index);
          if vertex_index == u16::MAX { continue; }
          Self::extract_quad(cell, cell_index, min_voxel_index, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
        }
      }
    }
  }

  // For every edge that crosses the isosurface, make a quad between the "centers" of the four cubes touching that 
  // surface. The "centers" are actually the vertex positions found earlier. Also make sure the triangles are facing the
  // right way. See the comments on `make_quad` to help with understanding the indexing.
  fn extract_quad(
    cell: Cell,
    cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = chunk_sample_array.sample_index(min_voxel_index).is_sign_negative();
    // Do edges parallel with the X axis
    if cell.y != 0 && cell.z != 0 && cell.x < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + Self::ADD_X_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        Self::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          cell_index,
          Self::ADD_Y_CELL_INDEX_OFFSET,
          Self::ADD_Z_CELL_INDEX_OFFSET,
        );
      }
    }
    // Do edges parallel with the Y axis
    if cell.x != 0 && cell.z != 0 && cell.y < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + Self::ADD_Y_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        Self::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          cell_index,
          Self::ADD_Z_CELL_INDEX_OFFSET,
          Self::ADD_X_CELL_INDEX_OFFSET,
        );
      }
    }
    // Do edges parallel with the Z axis
    if cell.x != 0 && cell.y != 0 && cell.z < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + Self::ADD_Z_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        Self::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          cell_index,
          Self::ADD_X_CELL_INDEX_OFFSET,
          Self::ADD_Y_CELL_INDEX_OFFSET,
        );
      }
    }
  }

  // Construct a quad in the dual graph of the SDF lattice.
  //
  // Surface point s was found somewhere inside of the cube (`cell_index`) with minimal corner p1 (`value_a_negative`).
  //
  //       x ---- x
  //      /      /|
  //     x ---- x |
  //     |   s  | x
  //     |      |/
  //    p1 --- p2
  //
  // And now we want to find the quad between p1 and p2 (`value_b_negative`) where s is a corner of the quad.
  //
  //          s
  //         /|
  //        / |
  //       |  |
  //   p1  |  |  p2
  //       | /
  //       |/
  //
  // A is (of the three grid axes) the axis between p1 and p2,
  //
  //       A
  //   p1 ---> p2
  //
  // therefore we must find the other 3 quad corners by moving along the other two axes (those orthogonal to A) in the 
  // negative directions; these are axis B and axis C.
  fn make_quad(
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
    value_a_negative: bool,
    value_b_negative: bool,
    cell_index: CellIndex,
    axis_b_cell_index_offset: CellIndex,
    axis_c_cell_index_offset: CellIndex,
  ) {
    let negative_face = match (value_a_negative, value_b_negative) {
      (true, false) => false,
      (false, true) => true,
      _ => unreachable!(),
    };

    // The triangle points, viewed face-front, look like this:
    // v1 v3
    // v2 v4
    let (v1, pos1) = Self::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, cell_index); // PERF: sharing this calculation decreases performance.
    let (v2, pos2) = Self::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, cell_index - axis_b_cell_index_offset);
    let (v3, pos3) = Self::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, cell_index - axis_c_cell_index_offset);
    let (v4, pos4) = Self::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, cell_index - axis_b_cell_index_offset - axis_c_cell_index_offset);
    // Split the quad along the shorter axis, rather than the longer one.
    let distance_a = (pos4 - pos1).mag_sq();
    let distance_b = (pos3 - pos2).mag_sq();
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
    chunk_mesh.extend_indices_from_slice(&quad);
  }

  #[inline]
  fn read_vertex_position(cell_index_to_vertex_index: &impl ArrayTrait<u16, CellIndex>, chunk_mesh: &ChunkMesh, cell_index: CellIndex) -> (u16, Vec3) {
    let vertex_index = cell_index_to_vertex_index.index(cell_index);
    debug_assert!(vertex_index < u16::MAX, "Tried to read vertex index that was not set in cell index to vertex index array, at cell index: {}", cell_index);
    let position = chunk_mesh.vertices()[vertex_index as usize].position;
    (vertex_index, position)
  }


  // Helpers

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
  pub fn global_voxel_positions(min: UVec3, step: u32, local_voxel_positions: &[UVec3; 8]) -> [Vec3; 8] {
    [
      Vec3::from(min + step * local_voxel_positions[0]),
      Vec3::from(min + step * local_voxel_positions[1]),
      Vec3::from(min + step * local_voxel_positions[2]),
      Vec3::from(min + step * local_voxel_positions[3]),
      Vec3::from(min + step * local_voxel_positions[4]),
      Vec3::from(min + step * local_voxel_positions[5]),
      Vec3::from(min + step * local_voxel_positions[6]),
      Vec3::from(min + step * local_voxel_positions[7]),
    ]
  }

  pub const EDGE_TO_VOXEL_INDICES: [u8; 12] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0100,
    0b0001_0011,
    0b0001_0101,
    0b0010_0011,
    0b0010_0110,
    0b0011_0111,
    0b0100_0101,
    0b0100_0110,
    0b0101_0111,
    0b0110_0111,
  ];

  const ADD_X_VOXEL_INDEX_OFFSET: VoxelIndex = VoxelIndex::unit_x::<C::VoxelChunkShape>();
  const ADD_Y_VOXEL_INDEX_OFFSET: VoxelIndex = VoxelIndex::unit_y::<C::VoxelChunkShape>();
  const ADD_Z_VOXEL_INDEX_OFFSET: VoxelIndex = VoxelIndex::unit_z::<C::VoxelChunkShape>();
  const ADD_X_CELL_INDEX_OFFSET: CellIndex = CellIndex::unit_x::<C::CellChunkShape>();
  const ADD_Y_CELL_INDEX_OFFSET: CellIndex = CellIndex::unit_y::<C::CellChunkShape>();
  const ADD_Z_CELL_INDEX_OFFSET: CellIndex = CellIndex::unit_z::<C::CellChunkShape>();
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
    CellShape::<C>::index_from_xyz(self.x, self.y, self.z)
  }

  #[inline]
  pub fn to_min_voxel_index<C: ChunkSize>(&self) -> VoxelIndex {
    VoxelShape::<C>::index_from_xyz(self.x, self.y, self.z)
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

  #[inline]
  pub fn to_index<C: ChunkSize>(&self) -> VoxelIndex {
    VoxelShape::<C>::index_from_xyz(self.x, self.y, self.z)
  }
}

