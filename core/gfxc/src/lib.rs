use std::{fs, io};
use std::error::Error;
use std::fs::{File, OpenOptions, read_to_string};
use std::io::{BufWriter, Write};
pub use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Context;
use bytemuck::cast_slice;
use naga::{Module, ShaderStage};
use naga::back::spv::{Options as SpvOptions, PipelineOptions as SpvPipelineOptions, write_vec as spv_to_vec, WriterFlags};
use naga::front::glsl::{Frontend as GlslFrontend, Options as GlslOptions};
use naga::valid::{Capabilities, ModuleInfo, ValidationFlags, Validator};
use regex::{Captures, Regex};
use walkdir::WalkDir;

pub fn compile_shaders() -> Result<(), Box<dyn Error>> {
  let source_directory = {
    let var = std::env::var_os("CARGO_MANIFEST_DIR")
      .context("CARGO_MANIFEST_DIR environment variable was not set")?;
    PathBuf::from(var).join("src")
  };
  let out_directory = {
    let var = std::env::var_os("OUT_DIR")
      .context("OUT_DIR environment variable was not set")?;
    PathBuf::from(var).join("shader")
  };

  let mut glsl_frontend = GlslFrontend::default();
  let mut validator = Validator::new(ValidationFlags::default(), Capabilities::default());

  for entry in WalkDir::new(&source_directory).into_iter().filter_map(|e| e.ok()) {
    let file_name = entry.file_name().to_string_lossy();
    if !file_name.ends_with(".glsl") { continue; }

    let source_file = entry.path();
    let relative_path = entry.path()
      .parent()
      .expect("Failed to get parent path")
      .strip_prefix(&source_directory)
      .expect("Failed to get relative path");

    let source_text = read_file_to_string(source_file)
      .with_context(|| format!("Failed to read source file '{}'", source_file.display()))?;
    let source_text = preprocess_includes(source_text, source_file)?;

    let (shader_stage, dest_file) = if file_name.ends_with("vert.glsl") {
      let name = file_name.replace("vert.glsl", "");
      let dest_file = out_directory.join(relative_path).join(format!("{}vert.spv", name));
      (ShaderStage::Vertex, dest_file)
    } else if file_name.ends_with("frag.glsl") {
      let name = file_name.replace("frag.glsl", "");
      let dest_file = out_directory.join(relative_path).join(format!("{}frag.spv", name));
      (ShaderStage::Fragment, dest_file)
    } else {
      continue;
    };

    let glsl_options = GlslOptions::from(shader_stage);
    let module = from_glsl(&mut glsl_frontend, &glsl_options, &source_text, source_file)?;

    let module_info = validator.validate(&module)
      .with_context(|| format!("Failed to validate GLSL source file '{}'", source_file.display()))?;

    let spv_options = {
      let mut options = SpvOptions::default();
      // Add flags
      options.flags |= WriterFlags::DEBUG;
      // Remove flags
      options.flags -= WriterFlags::ADJUST_COORDINATE_SPACE;
      options
    };
    let spv_pipeline_options = Some(SpvPipelineOptions {
      shader_stage,
      entry_point: "main".to_string(),
    });
    to_spv(&module, &module_info, &spv_options, spv_pipeline_options.as_ref(), &dest_file)?;
  }

  Ok(())
}

fn preprocess_includes(mut source_text: String, source_file: impl AsRef<Path>) -> anyhow::Result<String> {
  static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("#include \"(.+)\"").unwrap());

  let source_file = source_file.as_ref();
  if let Some(parent_directory) = source_file.parent() {
    let mut do_replace = true;
    while do_replace { // Yo dawg, replace includes in a loop, since includes can include includes.
      do_replace = false;
      source_text = REGEX.replace_all(&source_text, |c: &Captures| {
        do_replace = true; // Try to replace again if a replacement was made.
        let file_match = c.get(1).unwrap();
        let included_file = parent_directory.join(file_match.as_str());
        let included_text = read_file_to_string(&included_file)
          .with_context(|| format!("Failed to read included file '{}'", included_file.display())).unwrap();
        included_text
      }).to_string()
    }
  }
  Ok(source_text)
}

fn from_glsl(
  glsl_frontend: &mut GlslFrontend,
  options: &GlslOptions,
  source_text: &str,
  source_file: &Path,
) -> anyhow::Result<Module> {
  let module = glsl_frontend.parse(&options, source_text)
    .with_context(|| format!("Failed to parse GLSL source file '{}'", source_file.display()))?;
  Ok(module)
}

fn to_spv(
  module: &Module,
  module_info: &ModuleInfo,
  options: &SpvOptions,
  pipeline_options: Option<&SpvPipelineOptions>,
  dest_file: &Path,
) -> anyhow::Result<()> {
  let output = spv_to_vec(module, module_info, options, pipeline_options)
    .with_context(|| format!("Failed to compile module '{:?}' to SPIR-V", module))?;
  let mut writer = create_writer(dest_file)?;
  writer.write_all(cast_slice(&output))
    .with_context(|| format!("Failed to write bytes to destination file '{}'", dest_file.display()))?;
  Ok(())
}

fn read_file_to_string(file: &Path) -> Result<String, io::Error> {
  println!("cargo:rerun-if-changed={}", file.display());
  read_to_string(file)
}

fn create_writer(file: &Path) -> anyhow::Result<BufWriter<File>> {
  // Note: not using cargo:rerun-if-changed on the output file, since it would then always run due to all files being
  // regenerated again. Could use it if we compare modified times?
  if let Some(parent) = file.parent() {
    fs::create_dir_all(parent)
      .with_context(|| format!("Failed to recursively create directory '{}'", file.display()))?;
  }
  let writer = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(file)
    .with_context(|| format!("Failed to create writer for destination file '{}'", file.display()))?;
  let writer = BufWriter::new(writer);
  Ok(writer)
}
