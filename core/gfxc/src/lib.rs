use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
pub use std::path::Path;
use std::path::PathBuf;

pub use shaderc::{Compiler, ShaderKind};
use walkdir::WalkDir;
use shaderc::{CompileOptions, OptimizationLevel, ResolvedInclude, IncludeType};

pub fn compile_shaders() {
  let mut compiler = Compiler::new()
    .expect("Failed to initialize shader compiler");

  let root_input_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")
    .expect("CARGO_MANIFEST_DIR env variable was not set"))
    .join("src");
  let root_output_dir = PathBuf::from(std::env::var_os("OUT_DIR")
    .expect("OUT_DIR env variable was not set"))
    .join("shader");


  let compile_options = {
    let mut options = CompileOptions::new().unwrap();
    options.set_optimization_level(OptimizationLevel::Performance);
    options.set_include_callback(|include, ty, _source, _depth| {
      match ty {
        IncludeType::Relative => {
          let path = root_input_dir.join(include);
          read_file_to_string(&path).map_or_else(
            |e| Err(format!("Failed to read source file '{}' into a string: {:?}", path.display(), e).to_string()),
            |content| Ok(ResolvedInclude { resolved_name: path.to_string_lossy().to_string(), content }),
          )
        }
        IncludeType::Standard => {
          Err("Standard includes are not supported".to_string())
        }
      }
    });
    options
  };

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
      compiler.compile_shader(ShaderKind::Vertex, entry.path(), root_output_dir.join(relative_path).join(format!("{}.vert.spv", name)), Some(&compile_options));
    } else if file_name.ends_with(".frag.glsl") {
      let name = file_name.replace(".frag.glsl", "");
      compiler.compile_shader(ShaderKind::Fragment, entry.path(), root_output_dir.join(relative_path).join(format!("{}.frag.spv", name)), Some(&compile_options));
    }
  }
}

pub trait CompilerEx {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D, additional_options: Option<&CompileOptions>);
  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str, additional_options: Option<&CompileOptions>);
}

impl CompilerEx for Compiler {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D, additional_options: Option<&CompileOptions>) {
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();
    let source_text = read_file_to_string(src_path)
      .unwrap_or_else(|e| panic!("Failed to read source file '{}' into a string: {:?}", src_path.display(), e));
    let result = self.compile_into_spirv(
      &source_text,
      kind,
      src_path.file_name().map(|p| p.to_str().unwrap_or_default()).unwrap_or_default(),
      "main",
      additional_options,
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

  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str, additional_options: Option<&CompileOptions>) {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();
    self.compile_shader(ShaderKind::Vertex, src_dir.join(format!("{}.vert.glsl", name)), dst_dir.join(format!("{}.vert.spv", name)), additional_options);
    self.compile_shader(ShaderKind::Fragment, src_dir.join(format!("{}.frag.glsl", name)), dst_dir.join(format!("{}.frag.spv", name)), additional_options);
  }
}

fn read_file_to_string(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
  let path = path.as_ref();
  let mut reader = OpenOptions::new()
    .read(true)
    .open(path)?;
  let mut string = String::new();
  reader.read_to_string(&mut string)?;
  println!("cargo:rerun-if-changed={}", path.display());
  Ok(string)
}
