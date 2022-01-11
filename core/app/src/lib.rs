use std::sync::mpsc::Receiver;
use std::thread;

use dotenv;
use egui::{CtxRef, TopBottomPanel, Ui};
use thiserror::Error;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;
use wgpu::{Backends, CommandBuffer, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode, RequestAdapterOptions, RequestDeviceError, SurfaceError};

use common::input::RawInput;
use common::screen::{LogicalSize, ScreenSize};
use common::timing::{Duration, FrameTimer, TickTimer, TimingStats};
use gfx::{Frame, Gfx};
use gfx::prelude::*;
use gfx::surface::GfxSurface;
use gui::Gui;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{OsWindow, WindowCreateError};

use crate::debug_gui::DebugGui;

mod debug_gui;

#[derive(Debug)]
pub struct Os {
  pub window: OsWindow,
}

#[derive(Debug)]
pub struct Tick {
  pub time_target: Duration,
  pub count: u64,
}

pub struct GuiFrame {
  pub context: CtxRef,
}

impl std::ops::Deref for GuiFrame {
  type Target = CtxRef;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.context }
}

#[allow(unused_variables)]
pub trait Application {
  fn new(os: &Os, gfx: &Gfx) -> Self;


  fn screen_resize(&mut self, os: &Os, gfx: &Gfx, screen_size: ScreenSize) {}


  /// Return true to prevent the GUI from receiving keyboard events.
  fn is_capturing_keyboard(&self) -> bool { return false; }

  /// Return true to prevent the GUI from receiving mouse events.
  fn is_capturing_mouse(&self) -> bool { return false; }

  type Input;

  fn process_input(&mut self, input: RawInput) -> Self::Input;


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {}
  fn add_to_menu(&mut self, ui: &mut Ui) {}


  fn simulate(&mut self, tick: Tick, input: &Self::Input) {}

  /// Can return additional command buffers.
  fn render<'a>(&mut self, os: &Os, gfx: &Gfx, frame: Frame<'a>, gui_frame: &GuiFrame, input: &Self::Input) -> Box<dyn Iterator<Item=CommandBuffer>>;
}


pub struct Options {
  pub name: String,

  pub window_inner_size: LogicalSize,
  pub window_min_inner_size: LogicalSize,

  pub graphics_backends: Backends,
  pub graphics_adapter_power_preference: PowerPreference,
  pub require_graphics_device_features: Features,
  pub request_graphics_device_features: Features,
  pub graphics_device_limits: Limits,
  pub graphics_swap_chain_present_mode: PresentMode,
}

impl Default for Options {
  fn default() -> Self {
    let size = LogicalSize::new(1280.0, 720.0);
    Options {
      name: "GGG application".to_string(),

      window_inner_size: size,
      window_min_inner_size: size,

      graphics_backends: Backends::all(),
      graphics_adapter_power_preference: PowerPreference::LowPower,
      require_graphics_device_features: Features::empty(),
      request_graphics_device_features: Features::empty(),
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
    ..RequestAdapterOptions::default()
  }).await.ok_or(CreateError::AdapterRequestFail)?;

  let supported_features = adapter.features();
  let required_but_unsupported_features = options.require_graphics_device_features.difference(supported_features);
  if !required_but_unsupported_features.is_empty() {
    panic!("The following features were required but not supported: {:?}", required_but_unsupported_features);
  }
  let requested_but_unsupported_features = options.request_graphics_device_features.difference(supported_features);
  if !requested_but_unsupported_features.is_empty() {
    info!("The following features were requested but not supported: {:?}", requested_but_unsupported_features);
  }
  let requested_and_supported_features = options.request_graphics_device_features.intersection(supported_features);
  let requested_features = options.require_graphics_device_features.union(requested_and_supported_features);

  let (device, queue) = adapter.request_device(&DeviceDescriptor {
    features: requested_features,
    limits: options.graphics_device_limits,
    label: Some("Device"),
    ..DeviceDescriptor::default()
  }, None).await?;
  let screen_size = os.window.get_inner_size();
  let surface = GfxSurface::new(surface, &adapter, &device, options.graphics_swap_chain_present_mode, screen_size);
  let gui = Gui::new(&device, surface.get_texture_format());
  let gfx = Gfx { instance, adapter, device, queue, surface };

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
  let mut debug_gui = DebugGui::default();
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut timing_stats = TimingStats::new();
  let mut resized = false;
  let mut minimized = false;

  'main: loop {
    // Timing
    let frame_time = frame_timer.frame();
    timing_stats.frame(frame_time);
    tick_timer.update_lag(frame_time.delta);

    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        OsEvent::WindowResized(_) => resized = true,
        _ => {}
      }
    }

    // Recreate swap chain if needed
    if resized {
      let size = os.window.get_inner_size();
      if size.is_zero() {
        resized = false;
        minimized = true;
      } else {
        screen_size = size;
        gfx.surface = gfx.surface.resize(&gfx.adapter, &gfx.device, screen_size);
        app.screen_resize(&os, &gfx, screen_size);
        resized = false;
        minimized = false;
      }
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
    let gui_frame = if !minimized {
      let gui_context = gui.begin_frame(screen_size, frame_time.elapsed.as_s(), frame_time.delta.as_s());
      TopBottomPanel::top("GUI top panel").show(&gui_context, |ui| {
        app.add_to_menu(ui);
        egui::menu::bar(ui, |ui| {
          debug_gui.add_debug_menu(ui, |ui| app.add_to_debug_menu(ui));
        })
      });
      Some(GuiFrame { context: gui_context })
    } else {
      None
    };

    // Show input debugging GUI if enabled.
    if let Some(ref gui_frame) = gui_frame {
      debug_gui.show_input(&gui_frame, &raw_input);
    }

    // Let the application process input.
    let input = app.process_input(raw_input);

    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        let count = tick_timer.tick_start();
        let tick = Tick { count, time_target: tick_timer.time_target() };
        app.simulate(tick, &input);
        let tick_time = tick_timer.tick_end();
        timing_stats.tick(tick_time);
      }
    }
    let extrapolation = tick_timer.extrapolation();
    timing_stats.tick_lag(tick_timer.accumulated_lag(), extrapolation);

    // Skip rendering if minimized.
    if minimized { continue; }

    // Show timing debugging GUI if enabled.
    debug_gui.show_timing(gui_frame.as_ref().unwrap(), &timing_stats);

    // Get frame to draw into
    let surface_texture = match gfx.surface.get_current_texture() {
      Ok(surface_texture) => surface_texture,
      Err(e) => {
        match e {
          SurfaceError::Outdated => resized = true,
          SurfaceError::Lost => resized = true,
          SurfaceError::OutOfMemory => panic!("Allocating swap chain frame reported out of memory; stopping"),
          _ => {}
        };
        continue;
      }
    };
    if surface_texture.suboptimal {
      resized = true;
    }
    let output_texture = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Render
    let mut encoder = gfx.device.create_default_command_encoder();
    let frame = Frame {
      screen_size,
      output_texture: &output_texture,
      encoder: &mut encoder,
      extrapolation,
      time: frame_time,
    };
    let additional_command_buffers = app.render(&os, &gfx, frame, gui_frame.as_ref().unwrap(), &input);
    gui.render(screen_size, &gfx.device, &gfx.queue, &mut encoder, &output_texture);

    // Submit command buffers
    let command_buffer = encoder.finish();
    gfx.queue.submit(std::iter::once(command_buffer).chain(additional_command_buffers));

    // Present
    surface_texture.present()
  }
}
