#[macro_export]
macro_rules! shader_file {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/bin/", env!("CARGO_BIN_NAME"), "/", $p, ".spv") }
}

#[macro_export]
macro_rules! include_shader {
  ($p:expr) => { wgpu::include_spirv!($crate::shader_file!($p)) }
}

#[macro_export]
macro_rules! include_shader_without_validation {
  ($p:expr) => {
    wgpu::ShaderModuleDescriptor {
      label: Some($crate::shader_file!($p)),
      source: wgpu::util::make_spirv(include_bytes!($crate::shader_file!($p))),
      flags: wgpu::ShaderFlags::default(),
    }
  }
}
