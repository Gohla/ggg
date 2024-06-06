#![allow(dead_code)]

use std::path::{Path, PathBuf};

use tracing::Subscriber;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Default)]
pub struct TracingBuilder {
  console_filter: Option<EnvFilter>,
  log_file_path: Option<PathBuf>,
  file_filter: Option<EnvFilter>,
}
impl TracingBuilder {
  pub fn with_console_filter(mut self, console_filter: EnvFilter) -> Self {
    self.console_filter = Some(console_filter);
    self
  }

  pub fn with_log_file_path(mut self, log_file_path: impl ToOwned<Owned=PathBuf>) -> Self {
    self.log_file_path = Some(log_file_path.to_owned());
    self
  }
  pub fn with_log_file_path_opt(mut self, log_file_path: Option<impl ToOwned<Owned=PathBuf>>) -> Self {
    self.log_file_path = log_file_path.map(|p| p.to_owned());
    self
  }
  pub fn with_file_filter(mut self, file_filter: EnvFilter) -> Self {
    self.file_filter = Some(file_filter);
    self
  }

  pub fn build(self) -> Tracing {
    macro_rules! filter {
      ($env:literal) => {{
        EnvFilter::new(crate::env::run_or_compile_time_env!($env))
      }};
    }

    let console_filter = self.console_filter.unwrap_or_else(|| filter!("CONSOLE_LOG"));

    #[cfg(not(target_arch = "wasm32"))] {
      let file = self.log_file_path.as_ref().map(|p| (p.as_ref(), self.file_filter.unwrap_or_else(|| filter!("FILE_LOG"))));
      Tracing::new(console_filter, file)
    }
    #[cfg(target_arch = "wasm32")] {
      Tracing::new_wasm(console_filter)
    }
  }
}

pub struct Tracing {
  _file_layer_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl Tracing {
  #[cfg(not(target_arch = "wasm32"))]
  fn new(
    console_filter: EnvFilter,
    file: Option<(&Path, EnvFilter)>,
  ) -> Self {
    let registry = tracing_subscriber::registry();

    let console_layer = tracing_subscriber::fmt::layer()
      .with_writer(std::io::stderr)
      .with_filter(console_filter);
    let registry = registry.with(console_layer);

    let _file_layer_guard = if let Some((file_path, filter)) = file {
      if let Some((layer, _guard)) = file_tracing(file_path, filter) {
        let registry = registry.with(layer);
        registry.init();
        Some(_guard)
      } else {
        None
      }
    } else {
      None
    };

    Self { _file_layer_guard }
  }

  // TODO: rewrite WASM variant with changes to above code
  #[cfg(target_arch = "wasm32")]
  fn new_wasm(
    console_filter: EnvFilter,
  ) -> Self {
    let layered = tracing_subscriber::registry();
    let layered = layered.with(
      tracing_subscriber::fmt::layer()
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_ansi(false)
        .without_time()
        .with_filter(console_filter)
    );
    layered.init();

    let _file_tracing = FileTracing::default();
    Self { _file_tracing }
  }
}

fn file_tracing<S>(file_path: &Path, filter: EnvFilter) -> Option<(Box<dyn Layer<S> + Send + Sync + 'static>, tracing_appender::non_blocking::WorkerGuard)> where
  S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>
{
  use std::fs::{create_dir_all, File};
  use std::io::BufWriter;

  let result = (|| {
    if let Some(parent) = file_path.parent() {
      create_dir_all(parent)?;
    }
    File::create(file_path)
  })();

  match result {
    Err(e) => {
      tracing::error!("Cannot trace to file; could not truncate/create and open file '{}' for writing: {}", file_path.display(), e);
      None
    }
    Ok(file) => {
      let writer = BufWriter::new(file);
      let (non_blocking, guard) = tracing_appender::non_blocking(writer);
      let layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(filter)
        .boxed();
      Some((layer, guard))
    }
  }
}
