use std::sync::mpsc::Receiver;

use dotenv;
use egui::TopBottomPanel;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;
use wgpu::{DeviceDescriptor, Instance, RequestAdapterOptions, RequestDeviceError, SurfaceError};

use common::screen::ScreenSize;
use common::timing::{Duration, FrameTimer, TickTimer, TimingStats};
use gfx::{Frame, Gfx};
use gfx::prelude::*;
use gfx::surface::GfxSurface;
use gfx::texture::TextureBuilder;
use gui::Gui;
use os::context::OsContext;
use os::directory::Directories;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{OsWindow, WindowCreateError};

use crate::{Application, config, DebugGui, GuiFrame, Options, Os, Tick};

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

pub async fn run_async<A: Application + 'static>(options: Options) -> Result<(), CreateError> {
  #[cfg(target_arch = "wasm32")] {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
  }

  dotenv::dotenv().ok();

  let filter_layer = EnvFilter::from_default_env();
  let layered = tracing_subscriber::registry()
    .with(filter_layer)
    ;
  #[cfg(not(target_arch = "wasm32"))] {
    layered
      .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
      .init();
  }
  #[cfg(target_arch = "wasm32")] {
    layered
      .with(tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default()))
      .init();
  }

  let os_context = OsContext::new();
  let window = {
    OsWindow::new(&os_context, options.window_inner_size, options.window_min_inner_size, options.name.clone())?
  };

  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };
  let directories = Directories::new(&options.name);
  let os = Os { window, directories };

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
  let features = options.require_graphics_device_features.union(requested_and_supported_features);

  let supported_limits = adapter.limits();
  let limits = options.graphics_device_limits
    .using_resolution(supported_limits.clone())
    .using_alignment(supported_limits)
    ;

  let (device, queue) = adapter.request_device(&DeviceDescriptor {
    features,
    limits,
    label: Some("Device"),
    ..DeviceDescriptor::default()
  }, None).await?;
  let screen_size = os.window.get_inner_size();
  let surface = GfxSurface::new(surface, &adapter, &device, options.graphics_swap_chain_present_mode, screen_size);

  let gui = Gui::new(&device, surface.get_texture_format());

  let sample_count = options.sample_count;
  let depth_texture = options.depth_stencil_texture_format.map(|format| {
    TextureBuilder::new_depth(screen_size.physical, format)
      .with_sample_count(sample_count)
      .build(&device)
  });
  let multisampled_framebuffer = if options.sample_count > 1 {
    Some(TextureBuilder::new_multisampled_framebuffer(&surface, sample_count)
      .with_texture_label("Multisampling texture")
      .with_texture_view_label("Multisampling texture view")
      .build(&device)
    )
  } else { None };

  let gfx = Gfx { instance, adapter, device, queue, surface, depth_stencil_texture: depth_texture, multisampled_framebuffer, sample_count };

  let app_config = config::deserialize_app_config::<A>(os.directories.config_dir());

  run_app::<A>(options.name, os_context, os_event_sys, os_event_rx, os_input_sys, os, gfx, gui, screen_size, app_config)?;
}

// Native codepath

#[cfg(not(target_arch = "wasm32"))]
fn run_app<A: Application + 'static>(
  name: String,
  os_context: OsContext,
  os_event_sys: OsEventSys,
  os_event_rx: Receiver<OsEvent>,
  os_input_sys: OsInputSys,
  os: Os,
  gfx: Gfx,
  gui: Gui,
  screen_size: ScreenSize,
  app_config: A::Config,
) -> Result<!, CreateError> { // Run app loop in a new thread, while Winit takes over the current thread.
  let app_thread_join_handle = std::thread::Builder::new()
    .name(name)
    .spawn(move || {
      run_app_in_loop::<A>(os_event_rx, os_input_sys, os, gfx, gui, screen_size, app_config);
    })?;
  os_event_sys.run(os_context, app_thread_join_handle);
}

#[cfg(not(target_arch = "wasm32"))]
fn run_app_in_loop<A: Application>(
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  os: Os,
  mut gfx: Gfx,
  mut gui: Gui,
  mut screen_size: ScreenSize,
  app_config: A::Config,
) {
  let mut app = A::new(&os, &gfx, app_config);
  let mut debug_gui = DebugGui::default();
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut timing_stats = TimingStats::new();
  let mut resized = false;
  let mut minimized = false;

  loop {
    let stop = run_app_cycle(
      &os_event_rx,
      &mut os_input_sys,
      &os,
      &mut gfx,
      &mut gui,
      &mut screen_size,
      &mut app,
      &mut debug_gui,
      &mut frame_timer,
      &mut tick_timer,
      &mut timing_stats,
      &mut resized,
      &mut minimized,
    );
    if stop { break; }
  }

  config::serialize_app_config::<A>(os.directories.config_dir(), app.get_config());
}

// WASM codepath

#[cfg(target_arch = "wasm32")]
fn run_app<A: Application + 'static>(
  _name: String,
  os_context: OsContext,
  os_event_sys: OsEventSys,
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  os: Os,
  mut gfx: Gfx,
  mut gui: Gui,
  mut screen_size: ScreenSize,
  app_config: A::Config,
) -> Result<!, CreateError> {
  // We do not have control over the loop in WASM, and there are no threads. Let Winit take control and run app cycle
  // as part of the Winit event loop.
  let mut app = A::new(&os, &gfx, app_config);
  let mut debug_gui = DebugGui::default();
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut timing_stats = TimingStats::new();
  let mut resized = false;
  let mut minimized = false;
  os_event_sys.run(os_context, move || run_app_cycle(
    &os_event_rx,
    &mut os_input_sys,
    &os,
    &mut gfx,
    &mut gui,
    &mut screen_size,
    &mut app,
    &mut debug_gui,
    &mut frame_timer,
    &mut tick_timer,
    &mut timing_stats,
    &mut resized,
    &mut minimized,
  ));
}

// Shared codepath

fn run_app_cycle<A: Application>(
  os_event_rx: &Receiver<OsEvent>,
  os_input_sys: &mut OsInputSys,
  os: &Os,
  gfx: &mut Gfx,
  gui: &mut Gui,
  screen_size: &mut ScreenSize,
  app: &mut A,
  debug_gui: &mut DebugGui,
  frame_timer: &mut FrameTimer,
  tick_timer: &mut TickTimer,
  timing_stats: &mut TimingStats,
  resized: &mut bool,
  minimized: &mut bool,
) -> bool {
  // Timing
  let frame_time = frame_timer.frame();
  timing_stats.frame(frame_time);
  tick_timer.update_lag(frame_time.delta);

  // Process OS events
  for os_event in os_event_rx.try_iter() {
    match os_event {
      OsEvent::TerminateRequested => return true, // Stop the loop.
      OsEvent::WindowResized(_) => *resized = true,
      _ => {}
    }
  }

  // Recreate swap chain if needed
  if *resized {
    let size = os.window.get_inner_size();
    if size.is_zero() {
      *resized = false;
      *minimized = true;
    } else {
      *screen_size = size;
      gfx.resize_surface(*screen_size);
      app.screen_resize(&os, &gfx, *screen_size);
      *resized = false;
      *minimized = false;
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
  let gui_frame = if !*minimized {
    let gui_context = gui.begin_frame(*screen_size, frame_time.elapsed.as_s(), frame_time.delta.as_s());
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
  if *minimized { return false; }

  // Show timing debugging GUI if enabled.
  debug_gui.show_timing(gui_frame.as_ref().unwrap(), &timing_stats);

  // Get frame to draw into
  let surface_texture = match gfx.surface.get_current_texture() {
    Ok(surface_texture) => surface_texture,
    Err(e) => {
      match e {
        SurfaceError::Outdated => *resized = true,
        SurfaceError::Lost => *resized = true,
        SurfaceError::OutOfMemory => panic!("Allocating swap chain frame reported out of memory; stopping"),
        _ => {}
      };
      return false; // Skip rendering.
    }
  };
  if surface_texture.suboptimal {
    *resized = true;
  }
  let output_texture = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

  // Render
  let mut encoder = gfx.device.create_default_command_encoder();
  let frame = Frame {
    screen_size: *screen_size,
    output_texture: &output_texture,
    encoder: &mut encoder,
    extrapolation,
    time: frame_time,
  };
  let additional_command_buffers = app.render(&os, &gfx, frame, gui_frame.as_ref().unwrap(), &input);
  gui.render(*screen_size, &gfx.device, &gfx.queue, &mut encoder, &output_texture);

  // Submit command buffers
  let command_buffer = encoder.finish();
  gfx.queue.submit(std::iter::once(command_buffer).chain(additional_command_buffers));

  // Present
  surface_texture.present();

  return false; // Keep looping.
}
