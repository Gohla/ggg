#[macro_export]
macro_rules! shader_file {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/", $p, ".spv") }
}

#[macro_export]
macro_rules! include_shader {
  ($p:expr) => { wgpu::include_spirv!($crate::shader_file!($p)) }
}

#[macro_export]
macro_rules! include_shader_without_validation {
  ($p:expr) => {
    wgpu::ShaderModuleDescriptorSpirV {
      label: Some($crate::shader_file!($p)),
      source: wgpu::util::make_spirv_raw(include_bytes!($crate::shader_file!($p))),
    }
  }
}

#[macro_export]
macro_rules! shader_file_for_bin {
  ($p:expr) => { concat!(env!("OUT_DIR"), "/shader/bin/", env!("CARGO_BIN_NAME"), "/", $p, ".spv") }
}

#[macro_export]
macro_rules! include_shader_for_bin {
  ($p:expr) => { wgpu::include_spirv!($crate::shader_file_for_bin!($p)) }
}

#[macro_export]
macro_rules! include_shader_without_validation_for_bin {
  ($p:expr) => {
    wgpu::ShaderModuleDescriptorSpirV {
      label: Some($crate::shader_file!($p)),
      source: wgpu::util::make_spirv_raw(include_bytes!($crate::shader_file_for_bin!($p))),
    }
  }
}
