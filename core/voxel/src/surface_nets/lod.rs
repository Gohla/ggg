use std::marker::PhantomData;

use ultraviolet::{UVec3, Vec3};

use crate::chunk::index::{CellIndex, VoxelIndex};
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
    let x_a = C::CELLS_IN_CHUNK_ROW - 1;
    let mut cell_index_to_vertex_index_a = C::create_cell_chunk_deck_array(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      // Positive X of chunk A.
      Self::extract_global_positions_border_x(x_a, step, min_a, chunk_sample_array, &mut cell_index_to_vertex_index_a, chunk_mesh);
    }
    let x_b = 0;
    let mut cell_index_to_vertex_index_b = C::create_cell_chunk_deck_array(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      // Negative X of chunk B.
      Self::extract_global_positions_border_x(x_b, step, min_b, chunk_sample_array, &mut cell_index_to_vertex_index_b, chunk_mesh);
    }
    // Positive X of chunk A.
    Self::extract_quads_border_x(x_a, chunk_samples_a, &cell_index_to_vertex_index_a, chunk_samples_b, &cell_index_to_vertex_index_b, chunk_mesh);
    // Negative X of chunk B.
    Self::extract_quads_border_x(x_b, chunk_samples_a, &cell_index_to_vertex_index_a, chunk_samples_b, &cell_index_to_vertex_index_b, chunk_mesh);
  }


  // Extract positions

  fn extract_global_positions_border_x(
    x: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut C::CellsChunkDeckArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let cell = Cell::new(x, y, z);
        SurfaceNets::extract_global_position(cell, min, step, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }


  // Extract quads

  fn extract_quads_border_x(
    x: u32,
    chunk_samples_a: &ChunkSamples<C>,
    cell_index_to_vertex_index_a: &C::CellsChunkDeckArray<u16>,
    chunk_samples_b: &ChunkSamples<C>,
    cell_index_to_vertex_index_b: &C::CellsChunkDeckArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let cell = Cell::new(x, y, z);
        let cell_index = cell.to_index::<C>();
        let min_voxel_index = cell.to_min_voxel_index::<C>();
        let vertex_index = cell_index_to_vertex_index_a.index(cell_index.into_usize());
        if vertex_index == u16::MAX { continue; }
        Self::extract_quad_border_x(cell, cell_index, min_voxel_index, chunk_samples_a, cell_index_to_vertex_index_a, chunk_samples_b, cell_index_to_vertex_index_b, chunk_mesh);
      }
    }
  }

  fn extract_quad_border_x(
    cell: Cell,
    cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    chunk_samples_a: &ChunkSamples<C>,
    cell_index_to_vertex_index_a: &C::CellsChunkDeckArray<u16>,
    chunk_samples_b: &ChunkSamples<C>,
    cell_index_to_vertex_index_b: &C::CellsChunkDeckArray<u16>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let (value_a_negative, v1, pos1) = if cell.x == 0 {
      let negative = chunk_samples_b.sample_index(min_voxel_index).is_sign_negative();
      let (v1, pos1) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_b, chunk_mesh, cell_index); // OPTO: not sharing this computation may increase performance.  
      (negative, v1, pos1)
    } else {
      let negative = chunk_samples_a.sample_index(min_voxel_index).is_sign_negative();
      let (v1, pos1) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index); // OPTO: not sharing this computation may increase performance.  
      (negative, v1, pos1)
    };

    // Do edges parallel with the X axis
    if cell.y != 0 && cell.z != 0 && cell.x < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      // TODO: sample from the correct samples and with correct index
      let value_b_negative = chunk_samples_a.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_X_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        // TODO: sample from the correct samples and with correct index
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
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
    if cell.x != 0 && cell.z != 0 && cell.y < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      // TODO: sample from the correct samples and with correct index
      let value_b_negative = chunk_samples_a.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Y_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        // TODO: sample from the correct samples and with correct index
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Z_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
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
    if cell.x != 0 && cell.y != 0 && cell.z < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      // TODO: sample from the correct samples and with correct index
      let value_b_negative = chunk_samples_a.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Z_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        // TODO: sample from the correct samples and with correct index
        let (v2, pos2) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET);
        let (v3, pos3) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
        let (v4, pos4) = SurfaceNets::<C>::read_vertex_position(cell_index_to_vertex_index_a, chunk_mesh, cell_index - SurfaceNets::<C>::ADD_X_CELL_INDEX_OFFSET - SurfaceNets::<C>::ADD_Y_CELL_INDEX_OFFSET);
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
