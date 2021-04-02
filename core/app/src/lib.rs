use std::sync::mpsc::Receiver;
use std::thread;

use dotenv;
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;
use wgpu::SwapChainError;

use gfx::{AdapterRequestError, DeviceRequestError, GfxAdapter, GfxDevice, GfxInstance, GfxQueue, GfxSurface, GfxSwapChain};
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::{OsInputSys, RawInput};
use os::window::{OsWindow, WindowCreateError};
use util::timing::{Duration, FrameTime, FrameTimer, TickTimer};

pub trait App {
  fn new(
    window: &OsWindow,
    instance: &GfxInstance,
    surface: &GfxSurface,
    adapter: &GfxAdapter,
    device: &GfxDevice,
    queue: &GfxQueue,
    swap_chain: &GfxSwapChain,
  ) -> Self;

  fn process_input(
    &mut self,
    raw_input: RawInput,
  );

  fn simulate(
    &mut self,
    fixed_time_step: Duration,
  );

  fn render(
    &mut self,
    window: &OsWindow,
    instance: &GfxInstance,
    surface: &GfxSurface,
    adapter: &GfxAdapter,
    device: &GfxDevice,
    queue: &GfxQueue,
    swap_chain: &GfxSwapChain,
    frame_output_texture: &wgpu::SwapChainTexture,
    extrapolation: f64,
    frame_time: FrameTime,
  ) -> Box<dyn Iterator<Item=wgpu::CommandBuffer>>;
}


pub struct Options {
  inner_size: LogicalSize,
  min_inner_size: LogicalSize,
  name: String,
}

impl Default for Options {
  fn default() -> Self {
    let size = LogicalSize::new(1920f64, 1080f64);
    Options {
      inner_size: size,
      min_inner_size: size,
      name: "GGG application".to_string(),
    }
  }
}

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

pub fn run_with_defaults<A: App>(name: &str) -> Result<(), CreateError> {
  run::<A>(Options {
    name: name.into(),
    ..Options::default()
  })
}

pub fn run<A: App>(options: Options) -> Result<(), CreateError> {
  futures::executor::block_on(run_async::<A>(options))
}

pub async fn run_async<A: App>(options: Options) -> Result<(), CreateError> {
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
    OsWindow::new(&os_context, options.inner_size, options.min_inner_size, options.name.clone())?
  };
  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };

  let instance = GfxInstance::new_with_primary_backends();
  let surface = unsafe { instance.create_surface(window.get_inner()) };
  let adapter = instance.request_low_power_adapter(&surface).await?;
  let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
    ..wgpu::DeviceDescriptor::default()
  }, None).await?;
  let swap_chain = device.create_swap_chain_with_defaults(&surface, &adapter, window.get_inner_size());

  thread::Builder::new()
    .name(options.name)
    .spawn(move || {
      run_loop::<A>(window, os_event_rx, os_input_sys, instance, surface, adapter, device, queue, swap_chain);
    })?;
  os_event_sys.run(os_context); // Hijacks the current thread. All code after the this line is ignored!
  Ok(()) // Ignored, but needed to conform to the return type.
}


fn run_loop<A: App>(
  window: OsWindow,
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  instance: GfxInstance,
  surface: GfxSurface,
  adapter: GfxAdapter,
  device: GfxDevice,
  queue: GfxQueue,
  mut swap_chain: GfxSwapChain,
) {
  let mut app = A::new(
    &window,
    &instance,
    &surface,
    &adapter,
    &device,
    &queue,
    &swap_chain,
  );
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut recreate_swap_chain = false;

  'main: loop {
    // Timing
    let frame_time = frame_timer.frame();
    tick_timer.update_lag(frame_time.frame_time);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        _ => {}
      }
    }
    // Process input
    let raw_input = os_input_sys.update();
    app.process_input(raw_input);
    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        app.simulate(tick_timer.time_target());
        tick_timer.tick_end();
      }
    }
    // Recreate swap chain if needed
    if recreate_swap_chain {
      swap_chain = swap_chain.resize(&surface, &adapter, &device, window.get_inner_size());
      recreate_swap_chain = false;
    }
    // Get frame to draw into
    let frame = match swap_chain.get_current_frame() {
      Ok(frame) => frame,
      Err(e) => {
        match e {
          SwapChainError::Outdated => recreate_swap_chain = true,
          SwapChainError::Lost => recreate_swap_chain = true,
          SwapChainError::OutOfMemory => panic!("Allocating swap chain frame reported out of memory; stopping"),
          _ => {}
        };
        continue;
      }
    };
    if frame.suboptimal {
      recreate_swap_chain = true;
    }
    // Render
    let command_buffers = app.render(
      &window,
      &instance,
      &surface,
      &adapter,
      &device,
      &queue,
      &swap_chain,
      &frame.output,
      tick_timer.extrapolation(),
      frame_time,
    );
    // Submit command buffers
    queue.submit(command_buffers);
  }
}
