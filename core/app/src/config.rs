use std::path::Path;

use tracing::error;

use crate::Application;

pub(crate) fn deserialize_app_config<A: Application>(config_dir: &Path) -> A::Config {
  let path = config_dir.to_path_buf().join("config.ron");
  if let Ok(file) = std::fs::File::open(path) {
    let reader = std::io::BufReader::new(file);
    if let Ok(config) = ron::de::from_reader(reader) {
      config
    } else {
      A::Config::default()
    }
  } else {
    A::Config::default()
  }
}

pub(crate) fn serialize_app_config<A: Application>(config_dir: &Path, config: &A::Config) {
  let path = config_dir.to_path_buf().join("config.ron");
  std::fs::create_dir_all(config_dir).ok();
  match std::fs::File::create(path.clone()) {
    Ok(file) => {
      let writer = std::io::BufWriter::new(file);
      let pretty_config = ron::ser::PrettyConfig::default();
      match ron::ser::to_writer_pretty(writer, config, pretty_config) {
        Err(e) => error!("Cannot serialize application config; failed to serialize: {:?}", e),
        _ => {}
      }
    }
    Err(e) => error!("Cannot serialize application config; failed to create or truncate config file '{}': {:?}", path.display(), e)
  }
}
