pub mod size;
pub mod array;
pub mod shape;
pub mod index;
pub mod sample;
pub mod mesh;

// Value trait

pub trait Value: Copy + Send + Sync + 'static {}

impl<T> Value for T where T: Copy + Send + Sync + 'static {}
