//! Interface with the native operating system or web browser environment:
//!
//! - [Setup environment variables](env)
//! - [Setup tracing and outputting it to a console](tracing)
//! - [Get directories for reading/writing caches, logs, configuration, and data](directory)
//! - [Instantiate an OS context to create windows and handle OS events](context)
//! - [Create windows](windows)
//! - [Handle OS events](event)
//! - [Handle mouse and keyboard input](input)
//! - [Handle getting and setting clipboard text](clipboard)
//! - [Handle opening a URL with a web browser](open_url)

use std::sync::mpsc::Receiver;

use thiserror::Error;

use crate::directory::Directories;
use crate::event::{Event, EventLoop, EventLoopCreateError};
use crate::input::InputSys;
use crate::tracing::{Tracing, TracingBuilder};
use crate::window::WindowOptions;

pub mod env;
pub mod tracing;
pub mod directory;
pub mod window;
pub mod event;
pub mod input;
pub mod clipboard;
pub mod open_url;

// Light initialization

pub fn init() {
  #[cfg(feature = "profile-with-tracy")]
  tracy_client::Client::start();

  profiling::register_thread!();

  #[cfg(target_arch = "wasm32")]
  std::panic::set_hook(Box::new(console_error_panic_hook::hook));

  env::load_dotenv_into_env();
}

pub fn init_tracing() -> Tracing {
  TracingBuilder::default().build()
}


// Operating system (OS) interface

pub struct Os {
  pub directories: Directories,
  pub tracing: Tracing,
  pub event_rx: Receiver<Event>,
  pub input_sys: InputSys,
}


// OS interface creation

#[derive(Default, Clone, Debug)]
pub struct Options {
  pub application: ApplicationOptions,
  pub window: WindowOptions,
}

#[derive(Clone, Debug)]
pub struct ApplicationOptions {
  pub name: String,
  pub organization: String,
  pub qualifier: String,
}
impl Default for ApplicationOptions {
  fn default() -> Self {
    Self {
      name: "GGG application".to_string(),
      organization: "GGG".to_string(),
      qualifier: String::default(),
    }
  }
}

#[derive(Error, Debug)]
pub enum OsCreateError {
  #[error(transparent)]
  EventLoopCreateFail(#[from] EventLoopCreateError),
}

impl Os {
  pub fn new(mut options: Options) -> Result<(Self, EventLoop), OsCreateError> {
    init();

    if options.window.title.is_empty() {
      options.window.title = options.application.name.clone();
    }

    let directories = Directories::new(&options.application.name, &options.application.organization, &options.application.qualifier);

    let tracing = TracingBuilder::default()
      .with_log_file_path(directories.log_dir().join("log.txt"))
      .build();

    let (event_loop, event_rx, input_sys) = {
      let (event_loop, input_event_rx, event_rx) = EventLoop::new(options.window)?;
      let input_sys = InputSys::new(input_event_rx);
      (event_loop, event_rx, input_sys)
    };

    let os = Os {
      directories,
      tracing,
      event_rx,
      input_sys,
    };

    Ok((os, event_loop))
  }
}

