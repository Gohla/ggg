use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use ron::de::from_reader;
use ron::ser::{PrettyConfig, to_writer_pretty};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::error;

pub const CONFIG_FILE_PATH: &str = "config.ron";
pub const EGUI_FILE_PATH: &str = "egui.ron";

pub(crate) fn deserialize_config<T: DeserializeOwned + Default>(config_dir: &Path, config_file_name: &str) -> T {
  let path = config_dir.to_path_buf().join(config_file_name);
  if !path.exists() { return T::default(); }
  match File::open(&path) {
    Ok(file) => match from_reader(BufReader::new(file)) {
      Ok(config) => config,
      Err(e) => {
        error!("Cannot deserialize application config, returning default config; failed to deserialize from file '{}': {:?}", path.display(), e);
        T::default()
      }
    }
    Err(e) => {
      error!("Cannot deserialize application config, returning default config; failed to open file '{}': {:?}", path.display(), e);
      T::default()
    }
  }
}

pub(crate) fn serialize_config<T: Serialize>(config_dir: &Path, config_file_name: &str, config: &T) {
  let path = config_dir.to_path_buf().join(config_file_name);
  create_dir_all(config_dir).ok();
  match File::create(&path) {
    Ok(file) => {
      match to_writer_pretty(BufWriter::new(file), config, PrettyConfig::default()) {
        Err(e) => error!("Cannot serialize application config; failed to serialize to file '{}': {:?}", path.display(), e),
        _ => {}
      }
    }
    Err(e) => error!("Cannot serialize application config; failed to create or truncate config file '{}': {:?}", path.display(), e)
  }
}
