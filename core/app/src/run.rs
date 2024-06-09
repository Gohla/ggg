use std::sync::mpsc::{Receiver, RecvError, sync_channel};

use egui::TopBottomPanel;
use pollster::FutureExt;
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

/// Run application error.
#[derive(Error, Debug)]
pub enum RunError {
  #[error(transparent)]
  OsCreateFail(#[from] OsCreateError),
  #[error("Failed to create application thread: {0}")]
  ThreadCreateFail(#[from] std::io::Error),
  #[error(transparent)]
  EventLoopRunFail(#[from] EventLoopRunError),
}

/// Run an [application](A) using given `os_options` and `options`. Uses current thread as the main thread. Blocks this
/// main thread until the application stops or a panic occurs.
#[tracing::instrument(name = "main", skip_all, err)]
#[profiling::function]
pub fn run_main_thread<A: Application>(os_options: os::Options, options: Options) -> Result<(), RunError> {
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
    .spawn(move || { let _ = run_app_thread::<A>(os, app_thread_rx, options, event_loop_stopper); })?;

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


// Internals: Application thread function

#[tracing::instrument(name = "app", skip_all, err)]
#[profiling::function]
fn run_app_thread<A: Application>(
  os: Os,
  rx: Receiver<(Instance, Surface<'static>, Window)>,
  options: Options,
  event_loop_stopper: EventLoopStopper,
) -> Result<(), CreateError> {
  profiling::register_thread!();

  // Receive, from the event loop (main) thread: the graphics instance, surface, and window. Blocking until received.
  let (instance, surface, window) = rx.recv()?;
  tracing::trace!(?instance, ?surface, ?window, "Received data from event loop (main) thread; initializing application");

  // Create a runner and run it.
  let runner = Runner::<A>::new(os, instance, surface, window, options).block_on()?;
  runner.run();

  // Application has stopped: signal the event loop to stop.
  event_loop_stopper.stop()?;

  Ok(())
}


// Internals: runner

struct Runner<A> {
  os: Os,
  window: Window,
  gfx: Gfx,
  gui: Gui,

  app: A,
  debug_gui: DebugGui,
  frame_timer: FrameTimer,
  tick_timer: TickTimer,
  timing_stats: TimingStats,

  screen_size: ScreenSize,
  resized: bool,
  minimized: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct Config<A> {
  app_config: A,
  debug_gui: DebugGui,
}


// Create runner

#[derive(Error, Debug)]
enum CreateError {
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

impl<A: Application> Runner<A> {
  async fn new(os: Os, instance: Instance, surface: Surface<'static>, window: Window, options: Options) -> Result<Self, CreateError> {
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
    let app = A::new(&os, &gfx, screen_size, config.app_config);

    let run = Self {
      os,
      window,
      gfx,
      gui,

      app,
      debug_gui: config.debug_gui,
      frame_timer: FrameTimer::new(),
      tick_timer: TickTimer::new(Offset::from_ns(16_666_667)),
      timing_stats: TimingStats::new(),

      screen_size,
      resized: false,
      minimized: false,
    };
    Ok(run)
  }
}


// Run in a loop

impl<A: Application> Runner<A> {
  #[tracing::instrument(name = "loop", skip_all)]
  fn run(mut self) {
    loop {
      let stop = self.frame();
      profiling::finish_frame!();
      if stop { break; }
    }

    let config_dir = self.os.directories.config_dir();
    let config = Config { app_config: self.app.into_config(), debug_gui: self.debug_gui };
    config::serialize_config::<Config<A::Config>>(config_dir, &config::CONFIG_FILE_PATH, &config);
    self.gui.context.memory(|m| config::serialize_config::<egui::Memory>(config_dir, &config::EGUI_FILE_PATH, m));
  }

  #[profiling::function]
  #[tracing::instrument(name = "frame", skip_all)]
  fn frame(&mut self) -> bool {
    // Timing
    let frame_time = self.frame_timer.frame();
    self.timing_stats.frame(frame_time);
    self.tick_timer.update_lag(frame_time.delta);

    // Process OS events
    for event in self.os.event_rx.try_iter() {
      match event {
        Event::WindowCursor { cursor_in_window } => {
          self.gui.update_window_cursor(cursor_in_window);
        }
        Event::WindowFocus { window_has_focus } => {
          self.gui.update_window_focus(window_has_focus);
        }
        Event::WindowSizeChange(inner_size) => {
          self.screen_size = inner_size;
          if self.screen_size.is_zero() {
            self.minimized = true;
          } else {
            self.minimized = false;
          }
          self.resized = true;
        }
        Event::Stop => return true, // Stop the loop.
      }
    }

    // Recreate swap chain if needed
    if self.resized {
      if !self.minimized {
        self.gfx.resize_surface(self.screen_size);
        self.app.screen_resize(&self.os, &self.gfx, self.screen_size);
      }
      self.resized = false;
    }

    // Get raw input.
    let mut raw_input = self.os.input_sys.update();

    // If the app is capturing keyboard and/or mouse inputs, prevent the GUI from capturing those inputs.
    let gui_process_keyboard_events = !self.app.is_capturing_keyboard();
    let gui_process_mouse_events = !self.app.is_capturing_mouse();
    self.gui.process_input(&raw_input, gui_process_keyboard_events, gui_process_mouse_events);

    // If the GUI is capturing keyboard and/or mouse inputs, prevent the app from capturing those inputs.
    if gui_process_keyboard_events && self.gui.is_capturing_keyboard() {
      raw_input.remove_keyboard_input();
    }
    if gui_process_mouse_events && self.gui.is_capturing_mouse() {
      raw_input.remove_mouse_input();
    }

    // Create GUI frame
    let gui_frame = if !self.minimized {
      let gui_context = self.gui.begin_frame(self.screen_size, frame_time.elapsed.as_s(), frame_time.delta.as_s());
      TopBottomPanel::top("GUI top panel").show(&gui_context, |ui| {
        self.app.add_to_menu(ui);
        egui::menu::bar(ui, |ui| {
          self.debug_gui.add_debug_menu(ui, |ui| self.app.add_to_debug_menu(ui));
        })
      });
      Some(GuiFrame { context: gui_context })
    } else {
      None
    };

    // Show input debugging GUI if enabled.
    if let Some(ref gui_frame) = gui_frame {
      self.debug_gui.show_input(&gui_frame, &raw_input);
    }

    // Let the application process input.
    let input = self.app.process_input(raw_input);

    // Simulate tick
    if self.tick_timer.should_tick() {
      while self.tick_timer.should_tick() { // Run simulation.
        let count = self.tick_timer.tick_start();
        let tick = Tick { count, time_target: self.tick_timer.time_target() };
        self.app.simulate(tick, &input);
        let tick_time = self.tick_timer.tick_end();
        self.timing_stats.tick(tick_time);
      }
    }
    let extrapolation = self.tick_timer.extrapolation();
    self.timing_stats.tick_lag(self.tick_timer.accumulated_lag(), extrapolation);

    // Skip rendering if minimized.
    if self.minimized { return false; }

    // Show timing debugging GUI if enabled.
    self.debug_gui.show_timing(gui_frame.as_ref().unwrap(), &self.timing_stats);

    // Get swapchain texture to draw into and present.
    let surface_texture = match self.gfx.surface.get_current_texture() {
      Ok(surface_texture) => surface_texture,
      Err(cause) => {
        match cause {
          SurfaceError::Outdated => self.resized = true,
          SurfaceError::Lost => self.resized = true,
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
      self.resized = true;
    }
    let output_texture = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Render
    let mut encoder = self.gfx.device.create_default_command_encoder();
    let frame = Frame {
      screen_size: self.screen_size,
      output_texture: &output_texture,
      encoder: &mut encoder,
      extrapolation,
      time: frame_time,
    };
    let additional_command_buffers = self.app.render(&self.os, &self.gfx, frame, gui_frame.as_ref().unwrap(), &input);
    self.gui.render(&self.window, self.screen_size, &self.gfx.device, &self.gfx.queue, &mut encoder, &output_texture);

    // Submit command buffers
    let command_buffer = encoder.finish();
    self.gfx.queue.submit(std::iter::once(command_buffer).chain(additional_command_buffers));

    // Present
    surface_texture.present();

    return false; // Keep looping.
  }
}
