use std::sync::mpsc::Receiver;
use std::thread;

use thiserror::Error;
use tracing::debug;

use gfx::{AdapterRequestError, DeviceRequestError, GfxAdapter, GfxDevice, GfxInstance, GfxSurface, GfxSwapChain};
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{Window, WindowCreateError};
use util::timing::{Duration, FrameTime, FrameTimer, TickTimer};

// pub struct App {
//   window: Window,
//   os_event_sys: OsEventSys,
//   os_event_rx: Receiver<OsEvent>,
//   os_input_sys: OsInputSys,
//
//   instance: GfxInstance,
//   surface: GfxSurface,
//   adapter: GfxAdapter,
//   device: GfxDevice,
//   swap_chain: GfxSwapChain,
// }

// Ok(Self {
// window,
// os_event_sys,
// os_event_rx,
// os_input_sys,
//
// instance,
// surface,
// adapter,
// device,
// swap_chain,
// })

#[derive(Error, Debug)]
pub enum CreateError {
  #[error(transparent)]
  WindowCreateFail(#[from] WindowCreateError),
  #[error(transparent)]
  AdapterRequestFail(#[from] AdapterRequestError),
  #[error(transparent)]
  RequestDeviceFail(#[from] DeviceRequestError),
  #[error(transparent)]
  ThreadCreateFail(#[from] std::io::Error),
}

pub async fn run() -> Result<(), CreateError> {
  let os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(1920.0, 1080.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")?
  };
  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };

  let instance = GfxInstance::new_with_primary_backends();
  let surface = unsafe { instance.create_surface(window.winit_window()) };
  let adapter = instance.request_low_power_adapter(&surface).await?;
  let device = adapter.request_device(&wgpu::DeviceDescriptor {
    ..wgpu::DeviceDescriptor::default()
  }, None).await?;
  let swap_chain = device.create_swap_chain_with_defaults(&surface, &adapter, window.window_inner_size());

  thread::Builder::new()
    .name("Application".to_string())
    .spawn(move || {
      debug!("Application thread started");
      run_loop(window, os_event_rx, os_input_sys, instance, surface, adapter, device, swap_chain);
      debug!("Application thread stopped");
    })?;

  debug!("Main thread OS-event loop started");
  os_event_sys.run(os_context); // Hijacks the current thread. All code after the this line is ignored!
  Ok(()) // Ignored, but needed to conform to the return type.
}

fn run_loop(
  _window: Window,
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  _instance: GfxInstance,
  surface: GfxSurface,
  adapter: GfxAdapter,
  device: GfxDevice,
  mut swap_chain: GfxSwapChain,
) {
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
        OsEvent::WindowResized(screen_size) => {
          swap_chain = swap_chain.resize(&surface, &adapter, &device, screen_size);
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
    let frame = swap_chain.get_current_frame().unwrap().output;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });
    device.inner_queue().submit(std::iter::once(encoder.finish()));
  }
}
