use std::marker::PhantomData;

use ultraviolet::UVec3;

use crate::chunk::array::Array as ArrayTrait;
use crate::chunk::index::{CellIndex, VoxelIndex};
use crate::chunk::mesh::ChunkMesh;
use crate::chunk::sample::{ChunkSampleArray, ChunkSamples};
use crate::chunk::shape::Shape;
use crate::chunk::size::ChunkSize;
use crate::surface_nets::{Cell, SurfaceNets};

type ShapeX<C> = <C as ChunkSize>::CellDeckDoubleXShape;
type ShapeY<C> = <C as ChunkSize>::CellDeckDoubleYShape;
type ShapeZ<C> = <C as ChunkSize>::CellDeckDoubleZShape;
type Array<C> = <C as ChunkSize>::CellDeckDoubleArray<u16>;

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
    let mut cell_index_to_vertex_index = Array::<C>::new(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_global_positions_border_x(0, step, min_a, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_global_positions_border_x(1, step, min_b, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_quads_border_x(0, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_quads_border_x(1, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_y(
    &self,
    step: u32,
    min_a: UVec3,
    chunk_samples_a: &ChunkSamples<C>,
    min_b: UVec3,
    chunk_samples_b: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = Array::<C>::new(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_global_positions_border_y(0, step, min_a, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_global_positions_border_y(1, step, min_b, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_quads_border_y(0, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_quads_border_y(1, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_z(
    &self,
    step: u32,
    min_a: UVec3,
    chunk_samples_a: &ChunkSamples<C>,
    min_b: UVec3,
    chunk_samples_b: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = Array::<C>::new(u16::MAX);
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_global_positions_border_z(0, step, min_a, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_global_positions_border_z(1, step, min_b, chunk_sample_array, &mut cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_a {
      Self::extract_quads_border_z(0, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_b {
      Self::extract_quads_border_z(1, chunk_sample_array, &cell_index_to_vertex_index, chunk_mesh);
    }
  }


  // Extract positions

  fn extract_global_positions_border_x(
    x: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_x::<C>();
        if let Some(position) = SurfaceNets::<C>::extract_cell_vertex_positions(cell, min, step, chunk_sample_array) {
          let border_cell_index = border_cell.to_index::<ShapeX<C>>();
          SurfaceNets::<C>::write_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index, position);
        }
      }
    }
  }

  fn extract_global_positions_border_y(
    y: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_y::<C>();
        if let Some(position) = SurfaceNets::<C>::extract_cell_vertex_positions(cell, min, step, chunk_sample_array) {
          let border_cell_index = border_cell.to_index::<ShapeY<C>>();
          SurfaceNets::<C>::write_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index, position);
        }
      }
    }
  }

  fn extract_global_positions_border_z(
    z: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_z::<C>();
        if let Some(position) = SurfaceNets::<C>::extract_cell_vertex_positions(cell, min, step, chunk_sample_array) {
          let border_cell_index = border_cell.to_index::<ShapeZ<C>>();
          SurfaceNets::<C>::write_vertex_position(cell_index_to_vertex_index, chunk_mesh, border_cell_index, position);
        }
      }
    }
  }


  // Extract quads

  fn extract_quads_border_x(
    x: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeX<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        Self::extract_quad_border_x(border_cell, border_cell_index, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quads_border_y(
    y: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeY<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        Self::extract_quad_border_y(border_cell, border_cell_index, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quads_border_z(
    z: u32,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeZ<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        Self::extract_quad_border_z(border_cell, border_cell_index, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }


  #[inline]
  fn extract_quad_border_x(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let min_voxel_index = border_cell.to_cell_border_x::<C>().to_min_voxel_index::<C>();
    let value_a_negative = chunk_sample_array.sample_index(min_voxel_index).is_sign_negative();
    Self::extract_quad_parallel_y::<ShapeX<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_z::<ShapeX<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_y(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let min_voxel_index = border_cell.to_cell_border_y::<C>().to_min_voxel_index::<C>();
    let value_a_negative = chunk_sample_array.sample_index(min_voxel_index).is_sign_negative();
    Self::extract_quad_parallel_x::<ShapeY<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_z::<ShapeY<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_z(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let min_voxel_index = border_cell.to_cell_border_z::<C>().to_min_voxel_index::<C>();
    let value_a_negative = chunk_sample_array.sample_index(min_voxel_index).is_sign_negative();
    Self::extract_quad_parallel_x::<ShapeZ<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_y::<ShapeZ<C>>(border_cell, border_cell_index, min_voxel_index, value_a_negative, chunk_sample_array, cell_index_to_vertex_index, chunk_mesh);
  }


  #[inline]
  fn extract_quad_parallel_x<S: Shape<CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    value_a_negative: bool,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.y != 0 && border_cell.z != 0 && border_cell.x < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_X_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        SurfaceNets::<C>::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          border_cell_index,
          CellIndex::unit_y::<S>(),
          CellIndex::unit_z::<S>(),
        );
      }
    }
  }

  #[inline]
  fn extract_quad_parallel_y<S: Shape<CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    value_a_negative: bool,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.x != 0 && border_cell.z != 0 && border_cell.y < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Y_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        SurfaceNets::<C>::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          border_cell_index,
          CellIndex::unit_z::<S>(),
          CellIndex::unit_x::<S>(),
        );
      }
    }
  }

  #[inline]
  fn extract_quad_parallel_z<S: Shape<CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    min_voxel_index: VoxelIndex,
    value_a_negative: bool,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &Array<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.x != 0 && border_cell.y != 0 && border_cell.z < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = chunk_sample_array.sample_index(min_voxel_index + SurfaceNets::<C>::ADD_Z_VOXEL_INDEX_OFFSET).is_sign_negative();
      if value_a_negative != value_b_negative {
        SurfaceNets::<C>::make_quad(
          cell_index_to_vertex_index,
          chunk_mesh,
          value_a_negative,
          value_b_negative,
          border_cell_index,
          CellIndex::unit_x::<S>(),
          CellIndex::unit_y::<S>(),
        );
      }
    }
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
  pub fn to_index<S: Shape<CellIndex>>(&self) -> CellIndex {
    S::index_from_xyz(self.x, self.y, self.z)
  }

  #[inline]
  pub fn to_cell_border_x<C: ChunkSize>(&self) -> Cell {
    let x = (self.x + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(x, self.y, self.z)
  }

  #[inline]
  pub fn to_cell_border_y<C: ChunkSize>(&self) -> Cell {
    let y = (self.y + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(self.x, y, self.z)
  }

  #[inline]
  pub fn to_cell_border_z<C: ChunkSize>(&self) -> Cell {
    let z = (self.z + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(self.x, self.y, z)
  }
}
