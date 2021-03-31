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

fn main() -> Result<()> {
  // Load environment variables from .env file.
  dotenv::dotenv().ok();
  // Setup tracing
  let fmt_layer = fmt::layer()
    .with_writer(std::io::stderr)
    ;
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
  // Setup OS context, window, and event system.
  let os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(1920.0, 1080.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")
      .with_context(|| "Failed to create window")?
  };
  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };
  // Spawn application thread.
  let _app_thread = thread::Builder::new()
    .name("Triangle".to_string())
    .spawn(move || {
      debug!("Application thread started");
      run(window, os_event_rx, os_input_sys)
        .with_context(|| "Application thread stopped with an error").unwrap();
      debug!("Application thread stopped");
    })
    .with_context(|| "Failed to create game thread")?;
  // Run OS event system, hijacking the current thread. All code after the next line is ignored!
  debug!("Main thread OS-event loop started");
  os_event_sys.run(os_context);
  Ok(()) // Ignored, but needed to conform to the return type.
}

fn run(_window: Window, os_event_rx: Receiver<OsEvent>, mut os_input_sys: OsInputSys) -> Result<()> {
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  'main: loop {
    // Timing
    let FrameTime { frame_time, .. } = frame_timer.frame();
    tick_timer.update_lag(frame_time);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        OsEvent::WindowResized(_screen_size) => {
          // TODO: resize
          //gfx.screen_size_changed(screen_size);
        }
      }
    }
    // Process input
    let _raw_input = os_input_sys.update();
    // TODO: Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        //sim.simulate(tick_timer.time_target());
        tick_timer.tick_end();
      }
    }
    // TODO: Render frame
    //gfx.render_frame(&mut sim.world, camera_input, tick_timer.extrapolation(), frame_time)?;
  }

  Ok(())
}
