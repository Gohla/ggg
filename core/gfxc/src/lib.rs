use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
pub use std::path::Path;
use std::path::PathBuf;

pub use shaderc::{Compiler, ShaderKind};
use walkdir::WalkDir;

pub fn compile_shaders() {
  let mut compiler = Compiler::new()
    .expect("Failed to initialize shader compiler");
  let root_input_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")
    .expect("CARGO_MANIFEST_DIR env variable was not set"))
    .join("src");
  let root_output_dir = PathBuf::from(std::env::var_os("OUT_DIR")
    .expect("OUT_DIR env variable was not set"))
    .join("shader");
  for entry in WalkDir::new(root_input_dir.clone())
    .into_iter()
    .filter_map(|e| e.ok()) {
    let file_name = entry.file_name().to_string_lossy();
    if !file_name.ends_with(".glsl") { continue; }
    let relative_path = entry.path()
      .parent()
      .expect("Failed to get parent path")
      .strip_prefix(&root_input_dir)
      .expect("Failed to get relative path");
    if file_name.ends_with(".vert.glsl") {
      let name = file_name.replace(".vert.glsl", "");
      compiler.compile_shader(ShaderKind::Vertex, entry.path(), root_output_dir.join(relative_path).join(format!("{}.vert.spv", name)));
    } else if file_name.ends_with(".frag.glsl") {
      let name = file_name.replace(".frag.glsl", "");
      compiler.compile_shader(ShaderKind::Fragment, entry.path(), root_output_dir.join(relative_path).join(format!("{}.frag.spv", name)));
    }
  }
}

pub trait CompilerEx {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D);
  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str);
}

impl CompilerEx for Compiler {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D) {
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();
    let source_text = {
      let mut reader = OpenOptions::new()
        .read(true)
        .open(src_path)
        .unwrap_or_else(|e| panic!("Failed to create a reader for source file '{}': {:?}", src_path.display(), e));
      let mut string = String::new();
      reader.read_to_string(&mut string)
        .unwrap_or_else(|e| panic!("Failed to read source file '{}' into a string: {:?}", src_path.display(), e));
      println!("cargo:rerun-if-changed={}", src_path.display());
      string
    };
    let result = self.compile_into_spirv(
      &source_text,
      kind,
      src_path.file_name().map(|p| p.to_str().unwrap_or_default()).unwrap_or_default(),
      "main",
      None,
    ).unwrap_or_else(|e| panic!("Failed to compile shader file '{}': {:?}", src_path.display(), e));
    fs::create_dir_all(dst_path.parent().unwrap())
      .unwrap_or_else(|e| panic!("Failed to create destination directory '{}': {:}", dst_path.display(), e));
    let mut writer = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(dst_path)
      .unwrap_or_else(|e| panic!("Failed to create a writer for destination file '{}': {:?}", dst_path.display(), e));
    writer.write(result.as_binary_u8())
      .unwrap_or_else(|e| panic!("Failed to write bytes to destination file '{}': {:?}", dst_path.display(), e));
  }

  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str) {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();
    self.compile_shader(ShaderKind::Vertex, src_dir.join(format!("{}.vert.glsl", name)), dst_dir.join(format!("{}.vert.spv", name)));
    self.compile_shader(ShaderKind::Fragment, src_dir.join(format!("{}.frag.glsl", name)), dst_dir.join(format!("{}.frag.spv", name)));
  }
}
