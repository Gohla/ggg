use std::sync::mpsc::Receiver;
use std::thread;

use dotenv;
use thiserror::Error;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;
use wgpu::{Adapter, BackendBit, CommandBuffer, CommandEncoder, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions, RequestDeviceError, Surface, SwapChainError, SwapChainTexture};

use common::input::RawInput;
use common::timing::{Duration, FrameTime, FrameTimer, TickTimer};
use gfx::prelude::*;
use gfx::swap_chain::GfxSwapChain;
use gui::Gui;
pub use gui::GuiFrame;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{OsWindow, WindowCreateError};
use common::screen::{ScreenSize, LogicalSize};

#[derive(Debug)]
pub struct Os {
  pub window: OsWindow,
}

#[derive(Debug)]
pub struct Gfx {
  pub instance: Instance,
  pub surface: Surface,
  pub adapter: Adapter,
  pub device: Device,
  pub queue: Queue,
  pub swap_chain: GfxSwapChain,
}

#[derive(Debug)]
pub struct Tick {
  pub fixed_time_step: Duration
}

#[derive(Debug)]
pub struct Frame<'a> {
  pub screen_size: ScreenSize,
  pub output_texture: &'a SwapChainTexture,
  pub encoder: &'a mut CommandEncoder,
  pub extrapolation: f64,
  pub time: FrameTime,
}

pub trait Application {
  fn new(
    os: &Os,
    gfx: &Gfx,
  ) -> Self;


  #[allow(unused_variables)]
  fn screen_resize(
    &mut self,
    os: &Os,
    gfx: &Gfx,
    screen_size: ScreenSize,
  ) {}


  /// Return true to prevent the GUI from receiving keyboard events.
  fn is_capturing_keyboard(&self) -> bool { return false; }

  /// Return true to prevent the GUI from receiving mouse events.
  fn is_capturing_mouse(&self) -> bool { return false; }

  type Input;

  fn process_input(
    &mut self,
    gui_frame: &GuiFrame,
    input: RawInput,
  ) -> Self::Input;


  #[allow(unused_variables)]
  fn simulate(
    &mut self,
    tick: Tick,
    gui_frame: &GuiFrame,
    input: &Self::Input,
  ) {}

  fn render<'a>(
    &mut self,
    os: &Os,
    gfx: &Gfx,
    frame: Frame<'a>,
    gui_frame: &GuiFrame,
    input: &Self::Input,
  ) -> Box<dyn Iterator<Item=CommandBuffer>>; // Can return additional command buffers.
}


pub struct Options {
  name: String,

  window_inner_size: LogicalSize,
  window_min_inner_size: LogicalSize,

  graphics_backends: BackendBit,
  graphics_adapter_power_preference: PowerPreference,
  graphics_device_features: Features,
  graphics_device_limits: Limits,
  graphics_swap_chain_present_mode: PresentMode,
}

impl Default for Options {
  fn default() -> Self {
    let size = LogicalSize::new(1920f64, 1080f64);
    Options {
      name: "GGG application".to_string(),

      window_inner_size: size,
      window_min_inner_size: size,

      graphics_backends: BackendBit::all(),
      graphics_adapter_power_preference: PowerPreference::LowPower,
      graphics_device_features: Features::empty(),
      graphics_device_limits: Limits::default(),
      graphics_swap_chain_present_mode: PresentMode::Mailbox,
    }
  }
}

#[derive(Error, Debug)]
pub enum CreateError {
  #[error(transparent)]
  WindowCreateFail(#[from] WindowCreateError),
  #[error("Failed to request graphics adapter because no adapters were found that meet the required options")]
  AdapterRequestFail,
  #[error("Failed to request graphics device because no adapters were found that meet the required options")]
  RequestDeviceFail(#[from] RequestDeviceError),
  #[error(transparent)]
  ThreadCreateFail(#[from] std::io::Error),
}

pub fn run_with_defaults<A: Application>(name: &str) -> Result<(), CreateError> {
  run::<A>(Options {
    name: name.into(),
    ..Options::default()
  })
}

pub fn run<A: Application>(options: Options) -> Result<(), CreateError> {
  futures::executor::block_on(run_async::<A>(options))
}

pub async fn run_async<A: Application>(options: Options) -> Result<(), CreateError> {
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
    OsWindow::new(&os_context, options.window_inner_size, options.window_min_inner_size, options.name.clone())?
  };
  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };
  let os = Os { window };

  let instance = Instance::new(options.graphics_backends);
  let surface = unsafe { instance.create_surface(os.window.get_inner()) };
  let adapter = instance.request_adapter(&RequestAdapterOptions {
    power_preference: options.graphics_adapter_power_preference,
    compatible_surface: Some(&surface),
  }).await.ok_or(CreateError::AdapterRequestFail)?;
  let (device, queue) = adapter.request_device(&DeviceDescriptor {
    features: options.graphics_device_features,
    limits: options.graphics_device_limits,
    ..DeviceDescriptor::default()
  }, None).await?;
  let screen_size = os.window.get_inner_size();
  let swap_chain = GfxSwapChain::new(&surface, &adapter, &device, options.graphics_swap_chain_present_mode, screen_size);
  let gui = Gui::new(&device, swap_chain.get_texture_format());
  let gfx = Gfx { instance, surface, adapter, device, queue, swap_chain };

  thread::Builder::new()
    .name(options.name)
    .spawn(move || {
      run_loop::<A>(os_event_rx, os_input_sys, os, gfx, gui, screen_size);
    })?;
  os_event_sys.run(os_context); // Hijacks the current thread. All code after the this line is ignored!
  Ok(()) // Ignored, but needed to conform to the return type.
}


fn run_loop<A: Application>(
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  os: Os,
  mut gfx: Gfx,
  mut gui: Gui,
  mut screen_size: ScreenSize,
) {
  let mut app = A::new(&os, &gfx);
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut recreate_swap_chain = false;

  'main: loop {
    // Timing
    let frame_time = frame_timer.frame();
    tick_timer.update_lag(frame_time.delta);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        _ => {}
      }
    }

    // Recreate swap chain if needed
    if recreate_swap_chain {
      screen_size = os.window.get_inner_size();
      gfx.swap_chain = gfx.swap_chain.resize(&gfx.surface, &gfx.adapter, &gfx.device, screen_size);
      recreate_swap_chain = false;
      app.screen_resize(&os, &gfx, screen_size);
    }

    // Get raw input
    let mut raw_input = os_input_sys.update();

    // Let the GUI process input, letting the application prevent processing keyboard or mouse events if captured.
    let gui_process_keyboard_events = !app.is_capturing_keyboard();
    let gui_process_mouse_events = !app.is_capturing_mouse();
    gui.process_input(&raw_input, gui_process_keyboard_events, gui_process_mouse_events);

    // Then let the GUI prevent the application from processing keyboard or mouse events if captured.
    if gui_process_keyboard_events && gui.is_capturing_keyboard() {
      raw_input.remove_keyboard_input();
    }
    if gui_process_mouse_events && gui.is_capturing_mouse() {
      raw_input.remove_mouse_input();
    }

    // Create GUI frame
    let gui_frame = gui.begin_frame(screen_size, frame_time.elapsed.as_s(), frame_time.delta.as_s());

    // Let the application process input.
    let input = app.process_input(&gui_frame, raw_input);

    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        let tick = Tick { fixed_time_step: tick_timer.time_target() };
        app.simulate(tick, &gui_frame, &input);
        tick_timer.tick_end();
      }
    }

    // Get frame to draw into
    let swap_chain_frame = match gfx.swap_chain.get_current_frame() {
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
    if swap_chain_frame.suboptimal {
      recreate_swap_chain = true;
    }

    // Render
    let mut encoder = gfx.device.create_default_command_encoder();
    let frame = Frame {
      screen_size,
      output_texture: &swap_chain_frame.output,
      encoder: &mut encoder,
      extrapolation: tick_timer.extrapolation(),
      time: frame_time,
    };
    let additional_command_buffers = app.render(&os, &gfx, frame, &gui_frame, &input);
    gui.render(gui_frame, screen_size, &gfx.device, &gfx.queue, &mut encoder, &swap_chain_frame.output);

    // Submit command buffers
    let command_buffer = encoder.finish();
    gfx.queue.submit(std::iter::once(command_buffer).chain(additional_command_buffers));
  }
}
