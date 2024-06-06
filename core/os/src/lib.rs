//! Interface with the native operating system or web browser environment:
//!
//! - [Setup environment variables](env)
//! - [Setup tracing and outputting it to a console](tracing)
//! - [Get directories for reading/writing caches, logs, configuration, and data](directory)
//! - [Instantiate an OS context to create windows and handle OS events](context)
//! - [Create windows](windows)
//! - [Handle OS events](event)
//! - [Handle mouse and keyboard input](input)

use std::sync::mpsc::Receiver;

use thiserror::Error;

use common::screen::LogicalSize;

use crate::context::{Context, ContextCreateError};
use crate::directory::Directories;
use crate::event::{Event, EventLoopHandler, EventLoopRunner};
use crate::input::InputSys;
use crate::tracing::{Tracing, TracingBuilder};
use crate::window::{Window, WindowCreateError};

pub mod env;
pub mod tracing;
pub mod directory;
pub mod context;
pub mod window;
pub mod event;
pub mod input;

// Light initialization TODO: extract subparts into features and enable only needed features.

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
  pub options: Options,
  pub directories: Directories,
  pub tracing: Tracing,
  pub window: Window,
  pub event_rx: Receiver<Event>,
  pub input_sys: InputSys,
}

pub struct Options {
  pub name: String,
  pub window_inner_size: LogicalSize,
  pub window_min_inner_size: LogicalSize,
}
impl Default for Options {
  fn default() -> Self {
    #[cfg(not(target_arch = "wasm32"))]
      let size = LogicalSize::new(1920.0, 1080.0);
    #[cfg(target_arch = "wasm32")]
      let size = crate::window::get_browser_inner_size();
    Options {
      name: "GGG application".to_string(),
      window_inner_size: size,
      window_min_inner_size: size,
    }
  }
}


// OS interface creation

#[derive(Error, Debug)]
pub enum CreateError {
  #[error(transparent)]
  ContextCreateFail(#[from] ContextCreateError),
  #[error(transparent)]
  WindowCreateFail(#[from] WindowCreateError),
}

impl Os {
  pub fn new(options: Options) -> Result<(Self, EventLoopRunner), CreateError> {
    init();

    let directories = Directories::new(&options.name);

    let tracing = TracingBuilder::default()
      .with_log_file_path(directories.log_dir().join("log.txt"))
      .build();

    let context = Context::new()?;

    let window = Window::new(&context, options.window_inner_size, options.window_min_inner_size, options.name.clone())?;

    let (event_loop_handler, event_rx, input_sys) = {
      let (event_loop_handler, input_event_rx, event_rx) = EventLoopHandler::new(&window);
      let input_sys = InputSys::new(input_event_rx);
      (event_loop_handler, event_rx, input_sys)
    };
    let event_loop_runner = event_loop_handler.into_runner(context);

    let os = Os {
      options,
      directories,
      tracing,
      window,
      event_rx,
      input_sys,
    };

    Ok((os, event_loop_runner))
  }
}

