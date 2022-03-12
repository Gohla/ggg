// The following data originates from Eric Lengyel's Transvoxel Algorithm.
// http://transvoxel.org/

use ultraviolet::UVec3;

pub const REGULAR_VOXELS: [UVec3; 8] = [ // NOTE: Y and Z axis flipped!
  UVec3::new(0, 0, 0), // 0 (0, 0, 0)
  UVec3::new(1, 0, 0), // 1 (1, 0, 0)
  UVec3::new(0, 0, 1), // 2 (0, 1, 0)
  UVec3::new(1, 0, 1), // 3 (1, 1, 0)
  UVec3::new(0, 1, 0), // 4 (0, 0, 1)
  UVec3::new(1, 1, 0), // 5 (1, 0, 1)
  UVec3::new(0, 1, 1), // 6 (0, 1, 1)
  UVec3::new(1, 1, 1), // 7 (1, 1, 1)
];

/// Holds information about the triangulation used for a single equivalence class in the modified
/// Marching Cubes algorithm.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RegularCellData {
  /// High nibble is vertex count; low nibble is triangle count.
  geometry_counts: u8,
  /// Groups of 3 indices giving the triangulation.
  ///
  /// A value of `!0` is used for padding the array.
  pub vertex_index: [u8; 15],
}

impl RegularCellData {
  /// Gets the vertex count from `RegularCellData::geometry_counts`.
  pub fn get_vertex_count(&self) -> u8 {
    self.geometry_counts >> 4
  }

  /// Gets the triangle count from `RegularCellData::geometry_counts`.
  pub fn get_triangle_count(&self) -> u8 {
    self.geometry_counts & 0x0F
  }
}

/// Maps an 8-bit regular Marching Cubes case index to an equivalence class index.
pub const REGULAR_CELL_CLASS: [u8; 256] = [
  0x00, 0x01, 0x01, 0x03, 0x01, 0x03, 0x02, 0x04, 0x01, 0x02, 0x03, 0x04, 0x03, 0x04, 0x04, 0x03,
  0x01, 0x03, 0x02, 0x04, 0x02, 0x04, 0x06, 0x0C, 0x02, 0x05, 0x05, 0x0B, 0x05, 0x0A, 0x07, 0x04,
  0x01, 0x02, 0x03, 0x04, 0x02, 0x05, 0x05, 0x0A, 0x02, 0x06, 0x04, 0x0C, 0x05, 0x07, 0x0B, 0x04,
  0x03, 0x04, 0x04, 0x03, 0x05, 0x0B, 0x07, 0x04, 0x05, 0x07, 0x0A, 0x04, 0x08, 0x0E, 0x0E, 0x03,
  0x01, 0x02, 0x02, 0x05, 0x03, 0x04, 0x05, 0x0B, 0x02, 0x06, 0x05, 0x07, 0x04, 0x0C, 0x0A, 0x04,
  0x03, 0x04, 0x05, 0x0A, 0x04, 0x03, 0x07, 0x04, 0x05, 0x07, 0x08, 0x0E, 0x0B, 0x04, 0x0E, 0x03,
  0x02, 0x06, 0x05, 0x07, 0x05, 0x07, 0x08, 0x0E, 0x06, 0x09, 0x07, 0x0F, 0x07, 0x0F, 0x0E, 0x0D,
  0x04, 0x0C, 0x0B, 0x04, 0x0A, 0x04, 0x0E, 0x03, 0x07, 0x0F, 0x0E, 0x0D, 0x0E, 0x0D, 0x02, 0x01,
  0x01, 0x02, 0x02, 0x05, 0x02, 0x05, 0x06, 0x07, 0x03, 0x05, 0x04, 0x0A, 0x04, 0x0B, 0x0C, 0x04,
  0x02, 0x05, 0x06, 0x07, 0x06, 0x07, 0x09, 0x0F, 0x05, 0x08, 0x07, 0x0E, 0x07, 0x0E, 0x0F, 0x0D,
  0x03, 0x05, 0x04, 0x0B, 0x05, 0x08, 0x07, 0x0E, 0x04, 0x07, 0x03, 0x04, 0x0A, 0x0E, 0x04, 0x03,
  0x04, 0x0A, 0x0C, 0x04, 0x07, 0x0E, 0x0F, 0x0D, 0x0B, 0x0E, 0x04, 0x03, 0x0E, 0x02, 0x0D, 0x01,
  0x03, 0x05, 0x05, 0x08, 0x04, 0x0A, 0x07, 0x0E, 0x04, 0x07, 0x0B, 0x0E, 0x03, 0x04, 0x04, 0x03,
  0x04, 0x0B, 0x07, 0x0E, 0x0C, 0x04, 0x0F, 0x0D, 0x0A, 0x0E, 0x0E, 0x02, 0x04, 0x03, 0x0D, 0x01,
  0x04, 0x07, 0x0A, 0x0E, 0x0B, 0x0E, 0x0E, 0x02, 0x0C, 0x0F, 0x04, 0x0D, 0x04, 0x0D, 0x03, 0x01,
  0x03, 0x04, 0x04, 0x03, 0x04, 0x03, 0x0D, 0x01, 0x04, 0x0D, 0x03, 0x01, 0x03, 0x01, 0x01, 0x00,
];

/// Holds the triangulation data for all 16 distinct classes to which a case can be mapped by the
/// `REGULAR_CELL_CLASS` table.
///
/// A value of `!0` is used for padding the array.
pub const REGULAR_CELL_DATA: [RegularCellData; 16] = [
  RegularCellData { geometry_counts: 0x00, vertex_index: [!0; 15] },
  RegularCellData { geometry_counts: 0x31, vertex_index: [0, 1, 2, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x62, vertex_index: [0, 1, 2, 3, 4, 5, !0, !0, !0, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x42, vertex_index: [0, 1, 2, 0, 2, 3, !0, !0, !0, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x53, vertex_index: [0, 1, 4, 1, 3, 4, 1, 2, 3, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x73, vertex_index: [0, 1, 2, 0, 2, 3, 4, 5, 6, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x93, vertex_index: [0, 1, 2, 3, 4, 5, 6, 7, 8, !0, !0, !0, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x84, vertex_index: [0, 1, 4, 1, 3, 4, 1, 2, 3, 5, 6, 7, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x84, vertex_index: [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, !0, !0, !0] },
  RegularCellData { geometry_counts: 0xC4, vertex_index: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x64, vertex_index: [0, 4, 5, 0, 1, 4, 1, 3, 4, 1, 2, 3, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x64, vertex_index: [0, 5, 4, 0, 4, 1, 1, 4, 3, 1, 3, 2, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x64, vertex_index: [0, 4, 5, 0, 3, 4, 0, 1, 3, 1, 2, 3, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x64, vertex_index: [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, !0, !0, !0] },
  RegularCellData { geometry_counts: 0x75, vertex_index: [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6] },
  RegularCellData { geometry_counts: 0x95, vertex_index: [0, 4, 5, 0, 3, 4, 0, 1, 3, 1, 2, 3, 6, 7, 8] },
];

#[derive(Copy, Clone)]
pub struct RegularVertexData(pub u16);

impl RegularVertexData {
  #[inline]
  fn high_byte(&self) -> u8 { (self.0 >> 8) as u8 }
  #[inline]
  pub fn subtract_x(&self) -> bool {
    self.high_byte() & 0b0001_0000 != 0
  }
  #[inline]
  pub fn subtract_z(&self) -> bool { // NOTE: Z and Y axis flipped!
    self.high_byte() & 0b0010_0000 != 0
  }
  #[inline]
  pub fn subtract_y(&self) -> bool { // NOTE: Y and Z axis flipped!
    self.high_byte() & 0b0100_0000 != 0
  }
  #[inline]
  pub fn new_vertex(&self) -> bool {
    self.high_byte() & 0b1000_0000 != 0
  }
  #[inline]
  pub fn vertex_index(&self) -> u8 {
    self.high_byte() & 0b0000_1111 // Low nibble
  }

  #[inline]
  fn low_byte(&self) -> u8 { self.0 as u8 }
  #[inline]
  pub fn voxel_a_index(&self) -> u8 {
    self.low_byte() >> 4 // High nibble
  }
  #[inline]
  pub fn voxel_b_index(&self) -> u8 {
    self.low_byte() & 0b0000_1111 // Low nibble
  }
}

/// Gives the vertex locations for every one of the 256 possible cases in the modified Marching
/// Cubes algorithm. Each 16-bit value also provides information about whether a vertex can be
/// reused from a neighboring cell.
/// The low byte contains the indices for the two endpoints of the edge on which the vertex lies.
/// The high byte contains the vertex reuse data.
pub const REGULAR_VERTEX_DATA: [[u16; 12]; 256] = [
  [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x6201, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x1326, 0x3304, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2315, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x8337, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8337, 0x1326, 0x3304, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x8337, 0x1326, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2315, 0x8337, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x4113, 0x5102, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x1326, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x2245, 0x6201, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x2245, 0x6201, 0x2315, 0x4113, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x1326, 0x1146, 0x2245, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8337, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x2245, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x6201, 0x2315, 0x8337, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x2315, 0x2245, 0x1146, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x8337, 0x1326, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8337, 0x1326, 0x1146, 0x2245, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x8337, 0x1326, 0x5102, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x2315, 0x8337, 0x1326, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x6201, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x8157, 0x4113, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x1326, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x1326, 0x3304, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x8157, 0x4113, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x1326, 0x3304, 0x2245, 0x8157, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8337, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x4223, 0x4113, 0x8337, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x4223, 0x6201, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2245, 0x8157, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x8337, 0x1326, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8337, 0x1326, 0x3304, 0x6201, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x8337, 0x8157, 0x2245, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x8337, 0x1326, 0x3304, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x3304, 0x1146, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x8157, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x8157, 0x4113, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x5102, 0x1146, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x3304, 0x1146, 0x8157, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x4223, 0x6201, 0x2315, 0x8157, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x8157, 0x4113, 0x6201, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x1146, 0x8157, 0x4113, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x3304, 0x1146, 0x8157, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x8157, 0x2315, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x8157, 0x8337, 0x4223, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x5102, 0x1146, 0x8157, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x3304, 0x1146, 0x8157, 0x5102, 0x4113, 0x8337, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x8337, 0x1326, 0x1146, 0x8157, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x3304, 0x1146, 0x8157, 0x8337, 0x1326, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x1146, 0x8157, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2315, 0x4113, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x6201, 0x4223, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x8267, 0x1146, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x8267, 0x4223, 0x4113, 0x2315, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8337, 0x4223, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x4223, 0x4113, 0x8337, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x8337, 0x4223, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2315, 0x8337, 0x4223, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x1146, 0x5102, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x8337, 0x8267, 0x1146, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x8337, 0x8267, 0x1146, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x3304, 0x2315, 0x8337, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x8267, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x8267, 0x2245, 0x6201, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x8267, 0x2245, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x8267, 0x2245, 0x2315, 0x4113, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x8267, 0x2245, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x8267, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x8267, 0x2245, 0x3304, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x4223, 0x8267, 0x2245, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x8267, 0x2245, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x8267, 0x2245, 0x6201, 0x5102, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x8267, 0x2245, 0x4223, 0x6201, 0x2315, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x8267, 0x2245, 0x2315, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x8267, 0x8337, 0x4113, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8267, 0x2245, 0x6201, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x6201, 0x2315, 0x8337, 0x8267, 0x2245, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x8337, 0x8267, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8157, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x2315, 0x2245, 0x8157, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x8157, 0x4113, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x8157, 0x4113, 0x5102, 0x3304, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8267, 0x1146, 0x5102, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x6201, 0x4223, 0x8267, 0x1146, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8267, 0x1146, 0x5102, 0x6201, 0x2245, 0x8157, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x8157, 0x4113, 0x4223, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8337, 0x2315, 0x2245, 0x8157, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x4223, 0x4113, 0x8337, 0x2315, 0x2245, 0x8157, 0x1326, 0x8267, 0x1146],
  [0x8337, 0x4223, 0x6201, 0x2245, 0x8157, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x5102, 0x3304, 0x2245, 0x8157, 0x8337, 0x1326, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x1146, 0x5102, 0x4113, 0x8337, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x8337, 0x8267, 0x1146, 0x3304, 0x2315, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8267, 0x1146, 0x5102, 0x6201, 0x2245, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x8157, 0x8337, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x2315, 0x3304, 0x1326, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8157, 0x2315, 0x6201, 0x5102, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x1326, 0x3304, 0x6201, 0x4113, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8157, 0x4113, 0x5102, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x8267, 0x8157, 0x2315, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x6201, 0x4223, 0x8267, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x5102, 0x4223, 0x8267, 0x8157, 0x4113, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x4223, 0x8267, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x2315, 0x3304, 0x1326, 0x8267, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x2315, 0x6201, 0x5102, 0x1326, 0x8267, 0x4223, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x8337, 0x4223, 0x6201, 0x3304, 0x1326, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x8267, 0x8157, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8157, 0x2315, 0x3304, 0x5102, 0x4113, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x8337, 0x8267, 0x8157, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x3304, 0x5102, 0x8337, 0x8267, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8267, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x3304, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x3304, 0x2315, 0x4113, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x1326, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x1326, 0x3304, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x5102, 0x4223, 0x1326, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x1326, 0x3304, 0x2315, 0x4113, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8157, 0x8267, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8157, 0x8267, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x8267, 0x4223, 0x6201, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2315, 0x8157, 0x8267, 0x4223, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x5102, 0x4113, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x4113, 0x6201, 0x3304, 0x1326, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x5102, 0x6201, 0x2315, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x1326, 0x3304, 0x2315, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x3304, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x4113, 0x5102, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4223, 0x1326, 0x3304, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x2245, 0x6201, 0x4223, 0x1326, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2315, 0x4113, 0x5102, 0x4223, 0x1326, 0x3304, 0x1146, 0x2245, 0x8337, 0x8157, 0x8267],
  [0x4113, 0x4223, 0x1326, 0x1146, 0x2245, 0x2315, 0x8337, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8157, 0x8267, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x5102, 0x1146, 0x2245, 0x4223, 0x4113, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x8267, 0x4223, 0x6201, 0x2315, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x8157, 0x8267, 0x4223, 0x5102, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x5102, 0x4113, 0x8157, 0x8267, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x1146, 0x2245, 0x6201, 0x4113, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x6201, 0x2315, 0x8157, 0x8267, 0x1326, 0x3304, 0x1146, 0x2245, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x1146, 0x2245, 0x2315, 0x8157, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8267, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8267, 0x8337, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x6201, 0x2245, 0x8267, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x8337, 0x8267, 0x2245, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8267, 0x8337, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x1326, 0x3304, 0x8337, 0x2315, 0x2245, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x6201, 0x2245, 0x8267, 0x8337, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x4223, 0x1326, 0x3304, 0x2245, 0x8267, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8267, 0x4223, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x2245, 0x8267, 0x4223, 0x4113, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x8267, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x8267, 0x4223, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x2315, 0x2245, 0x8267, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x2315, 0x2245, 0x8267, 0x1326, 0x3304, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x6201, 0x2245, 0x8267, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x8267, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8337, 0x2315, 0x3304, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1146, 0x8267, 0x8337, 0x2315, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1146, 0x8267, 0x8337, 0x4113, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x4113, 0x5102, 0x1146, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8337, 0x2315, 0x3304, 0x1146, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x8267, 0x8337, 0x2315, 0x6201, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x8337, 0x4113, 0x6201, 0x3304, 0x1146, 0x5102, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x4223, 0x1326, 0x1146, 0x8267, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2315, 0x4113, 0x4223, 0x8267, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x6201, 0x5102, 0x1146, 0x8267, 0x4223, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x8267, 0x4223, 0x6201, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1146, 0x8267, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8267, 0x1326, 0x5102, 0x4113, 0x2315, 0x3304, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x2315, 0x1326, 0x1146, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x3304, 0x1146, 0x8267, 0x1326, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x1146, 0x8267, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x8337, 0x8157, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8157, 0x1146, 0x1326, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8157, 0x1146, 0x1326, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x5102, 0x3304, 0x2315, 0x1326, 0x8337, 0x8157, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8157, 0x1146, 0x5102, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x8337, 0x8157, 0x1146, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x8157, 0x1146, 0x5102, 0x4223, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x8157, 0x1146, 0x3304, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8157, 0x1146, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x4113, 0x8157, 0x1146, 0x1326, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x8157, 0x2315, 0x6201, 0x4223, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x5102, 0x3304, 0x2315, 0x8157, 0x1146, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8157, 0x1146, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x8157, 0x1146, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x8157, 0x1146, 0x5102, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x8157, 0x1146, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x3304, 0x1326, 0x8337, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x8157, 0x8337, 0x1326, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x3304, 0x1326, 0x8337, 0x8157, 0x6201, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x2315, 0x4113, 0x5102, 0x1326, 0x8337, 0x8157, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x8157, 0x2245, 0x3304, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x2245, 0x6201, 0x4223, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x3304, 0x5102, 0x4223, 0x8337, 0x8157, 0x4113, 0x6201, 0x2315, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x8157, 0x2245, 0x2315, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8157, 0x2245, 0x3304, 0x1326, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x4223, 0x4113, 0x8157, 0x2245, 0x6201, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8157, 0x2245, 0x3304, 0x1326, 0x4223, 0x6201, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x4223, 0x2315, 0x8157, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x5102, 0x4113, 0x8157, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x8157, 0x2245, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x6201, 0x2315, 0x8157, 0x2245, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2315, 0x8157, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x1326, 0x8337, 0x2315, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x1326, 0x8337, 0x2315, 0x2245, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x1146, 0x1326, 0x8337, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x1146, 0x1326, 0x8337, 0x4113, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1146, 0x2245, 0x2315, 0x8337, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x3304, 0x6201, 0x4223, 0x8337, 0x2315, 0x2245, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x8337, 0x4113, 0x6201, 0x2245, 0x1146, 0x5102, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x4113, 0x3304, 0x2245, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x2315, 0x2245, 0x1146, 0x1326, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1146, 0x1326, 0x4223, 0x4113, 0x2315, 0x2245, 0x6201, 0x5102, 0x3304, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x4223, 0x6201, 0x2245, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x5102, 0x3304, 0x2245, 0x1146, 0x1326, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x2245, 0x1146, 0x5102, 0x4113, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x2315, 0x2245, 0x1146, 0x3304, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x2245, 0x1146, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x2245, 0x1146, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x8337, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x8337, 0x2315, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x3304, 0x1326, 0x8337, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x8337, 0x4113, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4223, 0x8337, 0x2315, 0x3304, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4223, 0x8337, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x5102, 0x4223, 0x8337, 0x4113, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x4223, 0x8337, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x4113, 0x2315, 0x3304, 0x1326, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x1326, 0x4223, 0x4113, 0x2315, 0x6201, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x3304, 0x1326, 0x4223, 0x6201, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x1326, 0x4223, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x5102, 0x4113, 0x2315, 0x3304, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x4113, 0x2315, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x6201, 0x3304, 0x5102, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
  [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
];
