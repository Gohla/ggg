use gfxc::*;

fn main() {
  let mut compiler = Compiler::new().unwrap();
  let src_dir = Path::new("src");
  let dst_dir = Path::new("../../target/shader");
  compiler.compile_shader_pair(src_dir, dst_dir, "quad");
}
