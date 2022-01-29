#![feature(int_log)]

pub mod marching_cubes;
pub mod transvoxel;
pub mod volume;
pub mod octree;
pub mod chunk;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_SIZE_USIZE: usize = CHUNK_SIZE as usize;
