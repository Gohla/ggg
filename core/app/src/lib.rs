use std::sync::mpsc::Receiver;
use std::thread;

use dotenv;
use thiserror::Error;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;
use wgpu::SwapChainError;

use gfx::{AdapterRequestError, DeviceRequestError, GfxAdapter, GfxDevice, GfxInstance, GfxSurface, GfxSwapChain};
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{Window, WindowCreateError};
use util::timing::{Duration, FrameTime, FrameTimer, TickTimer};

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
  dotenv::dotenv().ok();

  let fmt_layer = fmt::layer()
    .with_writer(std::io::stderr)
    ;
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();

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
  let surface = unsafe { instance.create_surface(window.get_inner()) };
  let adapter = instance.request_low_power_adapter(&surface).await?;
  let device = adapter.request_device(&wgpu::DeviceDescriptor {
    ..wgpu::DeviceDescriptor::default()
  }, None).await?;
  let swap_chain = device.create_swap_chain_with_defaults(&surface, &adapter, window.get_inner_size());

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
  window: Window,
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
  let mut recreate_swapchain = false;
  'main: loop {
    if recreate_swapchain {
      swap_chain = swap_chain.resize(&surface, &adapter, &device, window.get_inner_size());
      recreate_swapchain = false;
    }

    // Timing
    let FrameTime { frame_time, .. } = frame_timer.frame();
    tick_timer.update_lag(frame_time);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        _ => {}
        // OsEvent::WindowResized(screen_size) => {
        //   //swap_chain = swap_chain.resize(&surface, &adapter, &device, screen_size);
        // }
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
    // Draw frame
    let frame = match swap_chain.get_current_frame() {
      Ok(frame) => frame,
      Err(e) => {
        match e {
          SwapChainError::Outdated => recreate_swapchain = true,
          SwapChainError::Lost => recreate_swapchain = true,
          SwapChainError::OutOfMemory => panic!("Allocating swap chain frame reported out of memory; stopping"),
          _ => {}
        };
        continue;
      }
    };
    if frame.suboptimal {
      recreate_swapchain = true;
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });
    {
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
          wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &frame.output.view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: true,
            },
          }
        ],
        depth_stencil_attachment: None,
      });
    }
    device.inner_queue().submit(std::iter::once(encoder.finish()));
  }
}
