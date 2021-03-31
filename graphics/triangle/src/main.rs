use anyhow::{Context, Result};

use math::prelude::*;
use os::context::OsContext;
use os::event_sys::OsEventSys;
use os::input_sys::OsInputSys;
use os::window::Window;

fn main() -> Result<()> {
  let mut os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(1920.0, 1080.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")
      .with_context(|| "Failed to create window")?
  };
  let (mut os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };
  os_event_sys.run(os_context);
  Ok(())
}
