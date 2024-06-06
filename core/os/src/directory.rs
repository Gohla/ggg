use std::path::Path;

use directories::ProjectDirs;

#[derive(Clone, Debug)]
pub struct Directories {
  project_dirs: ProjectDirs,
}

impl Directories {
  pub fn new(application_name: &str) -> Self {
    let project_dirs = ProjectDirs::from("", "GGG", application_name)
      .unwrap_or_else(|| panic!("Could not get project directories for application '{}'", application_name));
    Self { project_dirs }
  }

  #[inline]
  pub fn project_path(&self) -> &Path { self.project_dirs.project_path() }

  #[inline]
  pub fn cache_dir(&self) -> &Path { self.project_dirs.cache_dir() }

  #[inline]
  pub fn log_dir(&self) -> &Path { self.project_dirs.data_local_dir() }

  #[inline]
  pub fn config_dir(&self) -> &Path { self.project_dirs.config_dir() }

  #[inline]
  pub fn data_dir(&self) -> &Path { self.project_dirs.data_dir() }
}
