#[macro_export]
macro_rules! include_shader {
  ($p:expr) => { wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/shader/bin/", env!("CARGO_BIN_NAME"), "/", $p, ".spv")) }
}
