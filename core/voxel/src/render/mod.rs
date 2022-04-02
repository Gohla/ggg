// HACK: expose shaders here. This library should be responsible for rendering voxels instead?
pub fn get_vertex_shader() -> wgpu::ShaderModuleDescriptor<'static> { gfx::include_shader!("vert") }

pub fn get_fragment_shader() -> wgpu::ShaderModuleDescriptor<'static> { gfx::include_shader!("frag") }
