use std::sync::mpsc::Receiver;
use std::thread;

use anyhow::{Context, Result};
use dotenv;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;

use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::Window;
use util::timing::{Duration, FrameTime, FrameTimer, TickTimer};

fn main() {
  dotenv::dotenv().ok();
  let fmt_layer = fmt::layer()
    .with_writer(std::io::stderr)
    ;
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
  futures::executor::block_on(app::run()).unwrap();
}
