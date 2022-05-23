use std::marker::PhantomData;

use ultraviolet::UVec3;

use crate::chunk::array::Array;
use crate::chunk::index::CellIndex;
use crate::chunk::mesh::ChunkMesh;
use crate::chunk::sample::{ChunkSampleArray, ChunkSamples};
use crate::chunk::shape::Shape;
use crate::chunk::size::ChunkSize;
use crate::surface_nets::{Case, Cell, SurfaceNets};

type ShapeX<C> = <C as ChunkSize>::CellDeckDoubleXShape;
type ShapeY<C> = <C as ChunkSize>::CellDeckDoubleYShape;
type ShapeZ<C> = <C as ChunkSize>::CellDeckDoubleZShape;
type DeckVertexIndexArray<C> = <C as ChunkSize>::CellDeckDoubleArray<u16>;
type DeckCaseArray<C> = <C as ChunkSize>::CellDeckDoubleArray<Case>;
type ShapeXY<C> = <C as ChunkSize>::CellRowQuadXYShape;
type ShapeYZ<C> = <C as ChunkSize>::CellRowQuadYZShape;
type ShapeXZ<C> = <C as ChunkSize>::CellRowQuadXZShape;
type RowVertexIndexArray<C> = <C as ChunkSize>::CellRowQuadArray<u16>;
type RowCaseArray<C> = <C as ChunkSize>::CellRowQuadArray<Case>;

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
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_x: UVec3,
    chunk_samples_x: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = DeckVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = DeckCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_x(0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_x {
      Self::extract_global_positions_border_x(1, step, min_x, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_x(1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_y(
    &self,
    step: u32,
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_y: UVec3,
    chunk_samples_y: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = DeckVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = DeckCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_y(0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_y {
      Self::extract_global_positions_border_y(1, step, min_y, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_y(1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_z(
    &self,
    step: u32,
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_z: UVec3,
    chunk_samples_z: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = DeckVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = DeckCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_z(0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_z {
      Self::extract_global_positions_border_z(1, step, min_z, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_z(1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_xy(
    &self,
    step: u32,
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_x: UVec3,
    chunk_samples_x: &ChunkSamples<C>,
    min_y: UVec3,
    chunk_samples_y: &ChunkSamples<C>,
    min_xy: UVec3,
    chunk_samples_xy: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = RowVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = RowCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_xy(0, 0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_x {
      Self::extract_global_positions_border_xy(1, 0, step, min_x, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_y {
      Self::extract_global_positions_border_xy(0, 1, step, min_y, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_xy {
      Self::extract_global_positions_border_xy(1, 1, step, min_xy, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_xy(1, 1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_yz(
    &self,
    step: u32,
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_y: UVec3,
    chunk_samples_y: &ChunkSamples<C>,
    min_z: UVec3,
    chunk_samples_z: &ChunkSamples<C>,
    min_yz: UVec3,
    chunk_samples_yz: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = RowVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = RowCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_yz(0, 0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_y {
      Self::extract_global_positions_border_yz(1, 0, step, min_y, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_z {
      Self::extract_global_positions_border_yz(0, 1, step, min_z, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_yz {
      Self::extract_global_positions_border_yz(1, 1, step, min_yz, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_yz(1, 1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }

  #[profiling::function]
  pub fn extract_border_xz(
    &self,
    step: u32,
    min: UVec3,
    chunk_samples: &ChunkSamples<C>,
    min_x: UVec3,
    chunk_samples_x: &ChunkSamples<C>,
    min_z: UVec3,
    chunk_samples_z: &ChunkSamples<C>,
    min_xz: UVec3,
    chunk_samples_xz: &ChunkSamples<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let mut cell_index_to_vertex_index = RowVertexIndexArray::<C>::new(u16::MAX);
    let mut cell_index_to_case = RowCaseArray::<C>::new(Case::default());
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples {
      Self::extract_global_positions_border_xz(0, 0, step, min, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_x {
      Self::extract_global_positions_border_xz(1, 0, step, min_x, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_z {
      Self::extract_global_positions_border_xz(0, 1, step, min_z, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
    }
    if let ChunkSamples::Mixed(chunk_sample_array) = chunk_samples_xz {
      Self::extract_global_positions_border_xz(1, 1, step, min_xz, chunk_sample_array, &mut cell_index_to_vertex_index, &mut cell_index_to_case, chunk_mesh);
      Self::extract_quads_border_xz(1, 1, &cell_index_to_vertex_index, &cell_index_to_case, chunk_mesh);
    }
  }


  // Extract positions

  fn extract_global_positions_border_x(
    x: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut DeckVertexIndexArray<C>,
    cell_index_to_case: &mut DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_x::<C>();
        let border_cell_index = border_cell.to_index::<ShapeX<C>>();
        SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
      }
    }
  }

  fn extract_global_positions_border_y(
    y: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut DeckVertexIndexArray<C>,
    cell_index_to_case: &mut DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_y::<C>();
        let border_cell_index = border_cell.to_index::<ShapeY<C>>();
        SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
      }
    }
  }

  fn extract_global_positions_border_z(
    z: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut DeckVertexIndexArray<C>,
    cell_index_to_case: &mut DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let cell = border_cell.to_cell_border_z::<C>();
        let border_cell_index = border_cell.to_index::<ShapeZ<C>>();
        SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
      }
    }
  }

  fn extract_global_positions_border_xy(
    x: u32,
    y: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut RowVertexIndexArray<C>,
    cell_index_to_case: &mut RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let cell = border_cell.to_cell_border_xy::<C>();
      let border_cell_index = border_cell.to_index::<ShapeXY<C>>();
      SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
    }
  }

  fn extract_global_positions_border_yz(
    y: u32,
    z: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut RowVertexIndexArray<C>,
    cell_index_to_case: &mut RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for x in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let cell = border_cell.to_cell_border_yz::<C>();
      let border_cell_index = border_cell.to_index::<ShapeYZ<C>>();
      SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
    }
  }

  fn extract_global_positions_border_xz(
    x: u32,
    z: u32,
    step: u32,
    min: UVec3,
    chunk_sample_array: &ChunkSampleArray<C>,
    cell_index_to_vertex_index: &mut RowVertexIndexArray<C>,
    cell_index_to_case: &mut RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let cell = border_cell.to_cell_border_xz::<C>();
      let border_cell_index = border_cell.to_index::<ShapeXZ<C>>();
      SurfaceNets::<C>::extract_cell_vertex_positions(cell, border_cell_index, min, step, chunk_sample_array, cell_index_to_vertex_index, cell_index_to_case, chunk_mesh);
    }
  }


  // Extract quads

  fn extract_quads_border_x(
    x: u32,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    cell_index_to_case: &DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeX<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        let case = cell_index_to_case.index(border_cell_index);
        Self::extract_quad_border_x(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quads_border_y(
    y: u32,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    cell_index_to_case: &DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeY<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        let case = cell_index_to_case.index(border_cell_index);
        Self::extract_quad_border_y(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quads_border_z(
    z: u32,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    cell_index_to_case: &DeckCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      for x in 0..C::CELLS_IN_CHUNK_ROW {
        let border_cell = BorderCell::new(x, y, z);
        let border_cell_index = border_cell.to_index::<ShapeZ<C>>();
        let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
        if vertex_index == u16::MAX { continue; }
        let case = cell_index_to_case.index(border_cell_index);
        Self::extract_quad_border_z(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
      }
    }
  }

  fn extract_quads_border_xy(
    x: u32,
    y: u32,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    cell_index_to_case: &RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let border_cell_index = border_cell.to_index::<ShapeXY<C>>();
      let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
      if vertex_index == u16::MAX { continue; }
      let case = cell_index_to_case.index(border_cell_index);
      Self::extract_quad_border_xy(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
    }
  }

  fn extract_quads_border_yz(
    y: u32,
    z: u32,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    cell_index_to_case: &RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for x in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let border_cell_index = border_cell.to_index::<ShapeYZ<C>>();
      let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
      if vertex_index == u16::MAX { continue; }
      let case = cell_index_to_case.index(border_cell_index);
      Self::extract_quad_border_yz(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
    }
  }

  fn extract_quads_border_xz(
    x: u32,
    z: u32,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    cell_index_to_case: &RowCaseArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    for y in 0..C::CELLS_IN_CHUNK_ROW {
      let border_cell = BorderCell::new(x, y, z);
      let border_cell_index = border_cell.to_index::<ShapeXZ<C>>();
      let vertex_index = cell_index_to_vertex_index.index(border_cell_index);
      if vertex_index == u16::MAX { continue; }
      let case = cell_index_to_case.index(border_cell_index);
      Self::extract_quad_border_xz(border_cell, border_cell_index, case, cell_index_to_vertex_index, chunk_mesh);
    }
  }


  #[inline]
  fn extract_quad_border_x(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_y::<ShapeX<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_z::<ShapeX<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_y(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_x::<ShapeY<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_z::<ShapeY<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_z(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &DeckVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_x::<ShapeZ<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
    Self::extract_quad_parallel_y::<ShapeZ<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_xy(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_z::<ShapeXY<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_yz(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_x::<ShapeYZ<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }

  #[inline]
  fn extract_quad_border_xz(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    case: Case,
    cell_index_to_vertex_index: &RowVertexIndexArray<C>,
    chunk_mesh: &mut ChunkMesh,
  ) {
    let value_a_negative = case.is_min_negative();
    Self::extract_quad_parallel_y::<ShapeXZ<C>, _>(border_cell, border_cell_index, value_a_negative, case, cell_index_to_vertex_index, chunk_mesh);
  }


  // Attempt to extract a quad parallel to the X-axis. That is, a quad on the YZ plane.
  #[inline]
  fn extract_quad_parallel_x<S: Shape<CellIndex>, A: Array<u16, CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    value_a_negative: bool,
    case: Case,
    cell_index_to_vertex_index: &A,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.y != 0 && border_cell.z != 0 && border_cell.x < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = case.is_x_negative();
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

  // Attempt to extract a quad parallel to the Y-axis. That is, a quad on the ZX plane.
  #[inline]
  fn extract_quad_parallel_y<S: Shape<CellIndex>, A: Array<u16, CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    value_a_negative: bool,
    case: Case,
    cell_index_to_vertex_index: &A,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.x != 0 && border_cell.z != 0 && border_cell.y < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = case.is_y_negative();
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

  // Attempt to extract a quad parallel to the Z-axis. That is, a quad on the XY plane.
  #[inline]
  fn extract_quad_parallel_z<S: Shape<CellIndex>, A: Array<u16, CellIndex>>(
    border_cell: BorderCell,
    border_cell_index: CellIndex,
    value_a_negative: bool,
    case: Case,
    cell_index_to_vertex_index: &A,
    chunk_mesh: &mut ChunkMesh,
  ) {
    if border_cell.x != 0 && border_cell.y != 0 && border_cell.z < C::CELLS_IN_CHUNK_ROW { // PERF: removing the less-than check decreases performance.
      let value_b_negative = case.is_z_negative();
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

  #[inline]
  pub fn to_cell_border_xy<C: ChunkSize>(&self) -> Cell {
    let x = (self.x + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    let y = (self.y + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(x, y, self.z)
  }

  #[inline]
  pub fn to_cell_border_yz<C: ChunkSize>(&self) -> Cell {
    let y = (self.y + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    let z = (self.z + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(self.x, y, z)
  }

  #[inline]
  pub fn to_cell_border_xz<C: ChunkSize>(&self) -> Cell {
    let x = (self.x + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    let z = (self.z + C::CELLS_IN_CHUNK_ROW_MINUS_ONE) % C::CELLS_IN_CHUNK_ROW;
    Cell::new(x, self.y, z)
  }
}
