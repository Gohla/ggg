use std::marker::PhantomData;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::index::{CellIndex, ChunkIndices};
use crate::chunk::mesh::ChunkMesh;
use crate::chunk::sample::{ChunkSampleArray, ChunkSamples};
use crate::chunk::size::ChunkSize;
use crate::chunk::size::Sliceable;
use crate::surface_nets::{Cell, SurfaceNets};

#[derive(Default, Copy, Clone)]
pub struct SurfaceNetsLod<C: ChunkSize> {
  _chunk_size_phantom: PhantomData<C>,
}

impl<C: ChunkSize> SurfaceNetsLod<C> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  // Top-level functions

  #[profiling::function]
  pub fn extract_border_x(
    &self,
    step: u32,
    min_a: UVec3,
    chunk_samples_a: &ChunkSamples<C>,
    min_b: UVec3,
    chunk_samples_b: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = C::create_cell_chunk_deck_double_array(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_global_positions_border_x(BorderCell::border_part_a::<C>(), step, min_a, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_global_positions_border_x(BorderCell::border_part_b::<C>(), step, min_b, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_quads_border_x(BorderCell::border_part_a::<C>(), chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_quads_border_x(BorderCell::border_part_b::<C>(), chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
  }


  // Extract positions

  fn extract_global_positions_border_x(
    x: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut C::CellsChunkDeckDoubleArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        // TODO: CellsChunkDeckDoubleArray cannot be indexed by CellIndex, because it is only 2 x positions wide!
        let cell = Cell::new(x, y, z);
        SurfaceNets::extract_global_position(cell, min, step, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }


  // Extract quads

  fn extract_quads_border_x(
    x: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &C::CellsChunkDeckDoubleArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<C>();
        // TODO: CellsChunkDeckDoubleArray cannot be indexed by CellIndex, because it is only 2 x positions wide!
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index.into_usize());
        if vertex_index == u16::MAX { continue; }
        Self::extract_quad_border_x(border_cell, border_cell_index, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quad_border_x(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &C::CellsChunkDeckDoubleArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let cell = border_cell.to_cell_border_x::<C>();
    let min_voxel_index = cell.to_min_voxel_index::<C>();
    let value_a_negative = chunk_sample_array.sample_index(min_voxel_index).is_sign_negative();
    let (v1, pos1) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index); // OPTO: not sharing this computation may increase performance.

    // Do edges parallel with the X axis
    if border_cell.y != 0 && border_cell.z != 0 && border_cell.x < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_X_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
        Self::make_quad(
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          pos1, v1,
          pos2, v2,
          pos3, v3,
          pos4, v4,
        );
      }
    }
    // Do edges parallel with the Y axis
    if border_cell.x != 0 && border_cell.z != 0 && border_cell.y < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Y_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
        Self::make_quad(
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          pos1, v1,
          pos2, v2,
          pos3, v3,
          pos4, v4,
        );
      }
    }
    // Do edges parallel with the Z axis
    if border_cell.x != 0 && border_cell.y != 0 && border_cell.z < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Z_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
        Self::make_quad(
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          pos1, v1,
          pos2, v2,
          pos3, v3,
          pos4, v4,
        );
      }
    }
  }

  #[inline]
  fn make_quad(
    chunk_mesh: &mut ChunkMesh,
    value_a_negative: bool,
    value_b_negative: bool,
    pos1: Vec3, v1: u16,
    pos2: Vec3, v2: u16,
    pos3: Vec3, v3: u16,
    pos4: Vec3, v4: u16,
  ) {
    let negative_face = match (value_a_negative, value_b_negative) {
      (true, false) => false,
      (false, true) => true,
      _ => unreachable!(),
    };

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
}

/// Position of the minimal corner (left, bottom, back) of a border cell.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct BorderCell {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}

impl BorderCell {
  #[inline]
  pub const fn new(x: u32, y: u32, z: u32) -> Self {
    Self { x, y, z }
  }


  #[inline]
  pub fn to_index<C: ChunkSize>(&self) -> CellIndex {
    C::cell_index_from_xyz(self.x, self.y, self.z)
  }


  #[inline]
  pub fn border_part_a<C: ChunkSize>() -> u32 { C::CELLS_IN_CHUNK_ROW_MINUS_ONE }
  #[inline]
  pub fn border_part_b<C: ChunkSize>() -> u32 { 0 }


  #[inline]
  pub fn is_border_x_part_b<C: ChunkSize>(&self) -> bool { self.x == Self::border_part_b::<C>() }
  #[inline]
  pub fn select_border_x<'a, C: ChunkSize, T>(&self, a: &'a T, b: &'a T) -> &'a T {
    if self.is_border_x_part_b::<C>() { b } else { a }
  }
  #[inline]
  pub fn to_cell_border_x<C: ChunkSize>(&self) -> Cell {
    let x = (self.x + Self::border_part_a::<C>()) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(x, self.y, self.z)
  }
}
