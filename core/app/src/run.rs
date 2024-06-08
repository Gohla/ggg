use std::sync::mpsc::{Receiver, RecvError, sync_channel};

use egui::TopBottomPanel;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use wgpu::{CreateSurfaceError, DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceError};

use common::screen::ScreenSize;
use common::timing::{FrameTimer, Offset, TickTimer, TimingStats};
use gfx::{Frame, Gfx};
use gfx::prelude::*;
use gfx::surface::GfxSurface;
use gfx::texture::TextureBuilder;
use gui::Gui;
use os::event::{Event, EventLoopRunError, EventLoopStopError, EventLoopStopper};
use os::OsCreateError;
use os::window::Window;

use crate::{Application, config, DebugGui, GuiFrame, Options, Os, Tick};

#[derive(Error, Debug)]
pub enum RunError {
  #[error(transparent)]
  OsCreateFail(#[from] OsCreateError),
  #[error(transparent)]
  ThreadCreateFail(#[from] std::io::Error),
  #[error(transparent)]
  EventLoopRunFail(#[from] EventLoopRunError),
}

#[tracing::instrument(skip_all, err)]
#[profiling::function]
pub fn run<A: Application>(os_options: os::Options, options: Options) -> Result<(), RunError> {
  // Create the operating system interface.
  let (os, event_loop) = Os::new(os_options)?;

  // Create graphics instance now, since `options` is still in scope and not moved yet.
  let instance_descriptor = InstanceDescriptor {
    backends: options.graphics_backends,
    ..InstanceDescriptor::default()
  };
  let instance = Instance::new(instance_descriptor);

  // Create channel for synchronizing with the application thread.
  let (app_thread_tx, app_thread_rx) = sync_channel(1);

  // Run application in a thread.
  let event_loop_stopper = event_loop.create_event_loop_stopper();
  let app_thread_join_handle = std::thread::Builder::new()
    .name("Application Thread".to_string())
    .spawn(move || {
      profiling::register_thread!();
      let _ = run_app::<A>(os, app_thread_rx, options, event_loop_stopper);
    })?;

  // Make 1) the event loop stop when the application thread finishes, and 2) when the event loop stops, make it wait
  // for the application thread to finish.
  let event_loop = event_loop.with_join_handle(app_thread_join_handle);
  // Make the event loop run this callback on the event loop thread (main thread) when it creates the window.
  let event_loop = event_loop.with_on_window_created_callback(Box::new(move |window| {
    // Create the surface here on the event loop thread (main thread). This seems to be an undocumented requirement?
    let surface = instance.create_surface(window.cloned_winit_window())?;
    // Send the instance, surface, and window to the application thread.
    app_thread_tx.send((instance, surface, window))?;
    Ok(())
  }));

  // Run the event loop.
  event_loop.run()?;

  Ok(())
}


#[derive(Error, Debug)]
enum RunAppError {
  #[error("Failed to receive window from event loop: {0}")]
  ReceiveWindowFail(#[from] RecvError),
  #[error("Failed to create graphics surface: {0}")]
  CreateSurfaceFail(#[from] CreateSurfaceError),
  #[error("Failed to request graphics adapter because no adapters were found that meet the required options")]
  AdapterRequestFail,
  #[error("Failed to request graphics device because no adapters were found that meet the required options")]
  RequestDeviceFail(#[from] RequestDeviceError),
  #[error(transparent)]
  EventLoopStopFail(#[from] EventLoopStopError),
}

#[tracing::instrument(name = "app", skip_all, err)]
#[profiling::function]
fn run_app<A: Application>(
  os: Os,
  rx: Receiver<(Instance, Surface<'static>, Window)>,
  options: Options,
  event_loop_stopper: EventLoopStopper,
) -> Result<(), RunAppError> {
  use pollster::FutureExt;
  run_app_async::<A>(os, rx, options).block_on()?;
  event_loop_stopper.stop()?;
  Ok(())
}

async fn run_app_async<A: Application>(
  os: Os,
  rx: Receiver<(Instance, Surface<'static>, Window)>,
  options: Options,
) -> Result<(), RunAppError> {
  // Receive, from the event loop (main) thread: the graphics instance, surface, and window. Blocking until received.
  let (instance, surface, window) = rx.recv()?;
  tracing::trace!(?instance, ?surface, ?window, "Received data from event loop (main) thread");

  let adapter = instance.request_adapter(&RequestAdapterOptions {
    power_preference: options.graphics_adapter_power_preference,
    compatible_surface: Some(&surface),
    ..RequestAdapterOptions::default()
  }).await.ok_or(RunAppError::AdapterRequestFail)?;

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
  let required_features = options.require_graphics_device_features.union(requested_and_supported_features);

  let supported_limits = adapter.limits();
  let required_limits = options.graphics_device_limits
    .using_resolution(supported_limits.clone())
    .using_alignment(supported_limits)
    ;

  let (device, queue) = adapter.request_device(&DeviceDescriptor {
    required_features,
    required_limits,
    label: Some("Device"),
    ..DeviceDescriptor::default()
  }, None).await?;
  let screen_size = window.inner_size();
  let surface = GfxSurface::new(surface, &adapter, &device, options.graphics_swap_chain_present_mode, screen_size);
  tracing::debug!(configuration = ?surface.get_configuration(), "Created GFX surface");

  let egui_memory = config::deserialize_config::<egui::Memory>(os.directories.config_dir(), &config::EGUI_FILE_PATH);
  let gui = Gui::new(&device, surface.get_format(), Some(egui_memory));

  let sample_count = options.sample_count;
  let depth_stencil_texture = options.depth_stencil_texture_format.map(|format| {
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
  let gfx = Gfx { instance, adapter, device, queue, surface, depth_stencil_texture, multisampled_framebuffer, sample_count };

  let config = config::deserialize_config::<Config<A::Config>>(os.directories.config_dir(), &config::CONFIG_FILE_PATH);

  run_app_in_loop::<A>(os, window, gfx, gui, screen_size, config);

  Ok(())
}

#[derive(Default, Serialize, Deserialize)]
struct Config<A: Default> {
  app_config: A,
  debug_gui: DebugGui,
}


// Native codepath

#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(name = "loop", skip_all)]
fn run_app_in_loop<A: Application>(
  mut os: Os,
  window: Window,
  mut gfx: Gfx,
  mut gui: Gui,
  mut screen_size: ScreenSize,
  config: Config<A::Config>,
) {
  let mut app = A::new(&os, &gfx, screen_size, config.app_config);
  let mut debug_gui = config.debug_gui;
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Offset::from_ns(16_666_667));
  let mut timing_stats = TimingStats::new();
  let mut resized = false;
  let mut minimized = false;

  loop {
    let stop = run_app_cycle(
      &mut os,
      &window,
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

  let config = Config { app_config: app.into_config(), debug_gui };
  config::serialize_config::<Config<A::Config>>(os.directories.config_dir(), &config::CONFIG_FILE_PATH, &config);
  gui.context.memory(|m| config::serialize_config::<egui::Memory>(os.directories.config_dir(), &config::EGUI_FILE_PATH, m));
}

// WASM codepath

#[cfg(target_arch = "wasm32")]
fn run_app<A: Application + 'static>(
  _name: String,
  os_context: Context,
  os_event_sys: EventLoopHandler,
  os_event_rx: Receiver<Event>,
  mut os_input_sys: InputSys,
  os: Os,
  mut gfx: Gfx,
  mut gui: Gui,
  mut screen_size: ScreenSize,
  app_config: A::Config,
) -> Result<!, RunError> {
  // We do not have control over the loop in WASM, and there are no threads. Let Winit take control and run app cycle
  // as part of the Winit event loop.
  let mut app = A::new(&os, &gfx, app_config);
  let mut debug_gui = DebugGui::default();
  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Offset::from_ns(16_666_667));
  let mut timing_stats = TimingStats::new();
  let mut resized = false;
  let mut minimized = false;

  os_event_sys.run_event_loop(os_context, move || run_app_cycle(
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

#[tracing::instrument(name = "cycle", skip_all)]
#[profiling::function]
fn run_app_cycle<A: Application>(
  os: &mut Os,
  window: &Window,
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
  profiling::finish_frame!();

  // Timing
  let frame_time = frame_timer.frame();
  timing_stats.frame(frame_time);
  tick_timer.update_lag(frame_time.delta);

  // Process OS events
  for event in os.event_rx.try_iter() {
    match event {
      Event::WindowCursor { cursor_in_window } => {
        gui.update_window_cursor(cursor_in_window);
      }
      Event::WindowFocus { window_has_focus } => {
        gui.update_window_focus(window_has_focus);
      }
      Event::WindowSizeChange(inner_size) => {
        *screen_size = inner_size;
        if screen_size.is_zero() {
          *minimized = true;
        } else {
          *minimized = false;
        }
        *resized = true;
      }
      Event::Stop => return true, // Stop the loop.
    }
  }

  // Recreate swap chain if needed
  if *resized {
    if !*minimized {
      gfx.resize_surface(*screen_size);
      app.screen_resize(&os, &gfx, *screen_size);
    }
    *resized = false;
  }

  // Get raw input.
  let mut raw_input = os.input_sys.update();

  // If the app is capturing keyboard and/or mouse inputs, prevent the GUI from capturing those inputs.
  let gui_process_keyboard_events = !app.is_capturing_keyboard();
  let gui_process_mouse_events = !app.is_capturing_mouse();
  gui.process_input(&raw_input, gui_process_keyboard_events, gui_process_mouse_events);

  // If the GUI is capturing keyboard and/or mouse inputs, prevent the app from capturing those inputs.
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

  // Get swapchain texture to draw into and present.
  let surface_texture = match gfx.surface.get_current_texture() {
    Ok(surface_texture) => surface_texture,
    Err(cause) => {
      match cause {
        SurfaceError::Outdated => *resized = true,
        SurfaceError::Lost => *resized = true,
        SurfaceError::OutOfMemory => panic!("Getting next swapchain texture reported out of memory"),
        _ => {}
      }
      match cause {
        SurfaceError::Outdated => {}
        cause => tracing::warn!(?cause, "Failed to get next swapchain texture: {}", cause)
      }
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
  gui.render(window, *screen_size, &gfx.device, &gfx.queue, &mut encoder, &output_texture);

  // Submit command buffers
  let command_buffer = encoder.finish();
  gfx.queue.submit(std::iter::once(command_buffer).chain(additional_command_buffers));

  // Present
  surface_texture.present();

  return false; // Keep looping.
}
