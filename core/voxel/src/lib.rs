#![feature(int_log)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod marching_cubes;
pub mod transvoxel;
pub mod volume;
pub mod octree;
pub mod chunk;
pub mod uniform;

// HACK: expose shaders here. This library should be responsible for rendering voxels instead?
pub fn get_vertex_shader() -> wgpu::ShaderModuleDescriptor<'static> { gfx::include_shader!("vert") }

pub fn get_fragment_shader() -> wgpu::ShaderModuleDescriptor<'static> { gfx::include_shader!("frag") }
