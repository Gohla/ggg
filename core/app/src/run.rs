use std::sync::mpsc::{Receiver, RecvError, sync_channel};

use egui::TopBottomPanel;
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, trace_span};
use wgpu::{CreateSurfaceError, DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceError};

use common::screen::ScreenSize;
use common::time::{Offset, Stopwatch};
use egui_integration::GuiIntegration;
use gfx::{Gfx, GfxFrame};
use gfx::prelude::*;
use gfx::surface::GfxSurface;
use gfx::texture::TextureBuilder;
use gui::Gui;
use os::event::{Event, EventLoopRunError, EventLoopStopError, EventLoopStopper};
use os::OsCreateError;
use os::window::Window;

use crate::{Application, config, DebugGui, Frame, Options, Os, RenderInput, Step};
use crate::debug_gui::TimingStats;

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
    .name("Application thread".to_string())
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
  gui_integration: GuiIntegration,

  app: A,
  debug_gui: DebugGui,
  stopwatch: Stopwatch,
  frames: Frames,
  updates: Updates,
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
    let gui = GuiIntegration::new(Some(egui_memory), &device, surface.get_swapchain_texture_format());

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
    let gfx = Gfx { instance, adapter, device, queue, surface, depth_stencil_texture, multisample_output_texture: multisampled_framebuffer, sample_count };

    let config = config::deserialize_config::<Config<A::Config>>(os.directories.config_dir(), &config::CONFIG_FILE_PATH);
    let app = A::new(&os, &gfx, screen_size, config.app_config);

    let run = Self {
      os,
      window,
      gfx,
      gui_integration: gui,

      app,
      debug_gui: config.debug_gui,
      stopwatch: Stopwatch::default(),
      frames: Frames::default(),
      updates: Updates::new(Offset::from_nanoseconds(16_666_667)),
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
    let mut frame_end = FrameEnd::default();
    loop {
      let frame = self.frames.start(frame_end);
      let _frame_span = trace_span!("frame", frame = frame.frame).entered();
      self.timing_stats.frame_start(frame);

      let stop = self.frame(frame);

      frame_end = self.frames.end();
      self.updates.accumulate_lag(frame_end.duration);
      self.timing_stats.frame_end(frame_end);

      profiling::finish_frame!();
      if stop { break; }
    }

    let config_dir = self.os.directories.config_dir();
    let config = Config { app_config: self.app.into_config(), debug_gui: self.debug_gui };
    config::serialize_config::<Config<A::Config>>(config_dir, &config::CONFIG_FILE_PATH, &config);
    self.gui_integration.into_context().memory(|m| config::serialize_config::<egui::Memory>(config_dir, &config::EGUI_FILE_PATH, m));
  }

  #[profiling::function]
  fn frame(&mut self, frame: Frame) -> bool {
    // Elapsed time
    let elapsed = self.stopwatch.elapsed();
    self.timing_stats.elapsed(elapsed);

    // Process OS events
    for event in self.os.event_rx.try_iter() {
      match event {
        Event::WindowCursor { cursor_in_window } => {
          self.gui_integration.process_window_cursor_event(cursor_in_window);
        }
        Event::WindowFocus { window_has_focus } => {
          self.gui_integration.process_window_focus_event(window_has_focus);
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
    let gui_process_keyboard = !self.app.wants_keyboard_input();
    let gui_process_mouse = !self.app.wants_mouse_input();
    self.gui_integration.process_input(&raw_input, gui_process_keyboard, gui_process_mouse);

    // Create GUI frame
    let gui = if !self.minimized {
      let context = self.gui_integration.begin_frame(self.screen_size, elapsed.into_seconds(), frame.duration.into_seconds() as f32);

      // If the GUI is capturing keyboard and/or mouse inputs, prevent the app from capturing those inputs.
      if gui_process_keyboard && context.wants_keyboard_input() {
        raw_input.remove_keyboard_input();
      }
      if gui_process_mouse && context.wants_pointer_input() {
        raw_input.remove_mouse_input();
      }

      TopBottomPanel::top("GUI top panel").show(&context, |ui| {
        self.app.add_to_menu(ui);
        egui::menu::bar(ui, |ui| {
          self.debug_gui.add_debug_menu(ui, |ui| self.app.add_to_debug_menu(ui));
        })
      });
      let area_under_title_bar = context.available_rect();
      Some(Gui { context, area_under_title_bar })
    } else {
      None
    };

    // Show input debugging GUI if enabled.
    if let Some(gui) = gui.as_ref() {
      self.debug_gui.show_input(&gui, &raw_input);
    }

    // Let the application process input.
    let input = self.app.process_input(raw_input);

    // Simulate
    self.timing_stats.update_start(&self.updates);
    while self.updates.should_step() { // Run simulation steps.
      let step = self.updates.start_step();
      let _step_span = trace_span!("step", step = step.update).entered();
      self.timing_stats.step_start(step);
      self.app.simulate(step, &input);
      let step_end = self.updates.end_step();
      self.timing_stats.step_end(step_end);
    }
    self.timing_stats.update_end(&self.updates);

    // Skip rendering if minimized.
    if self.minimized { return false; }

    // Show timing debugging GUI if enabled.
    self.debug_gui.show_timing(gui.as_ref().unwrap(), &self.timing_stats);

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

    // Render application
    let encoder = self.gfx.device.create_default_command_encoder();
    let mut gfx_frame = GfxFrame {
      gfx: &self.gfx,
      screen_size: self.screen_size,
      output_texture,
      encoder,
    };
    let render_input = RenderInput {
      os: &self.os,
      gfx: &self.gfx,
      elapsed,
      frame,
      gfx_frame: &mut gfx_frame,
      extrapolate: self.updates.accumulated_lag,
      gui: gui.unwrap(),
      input: &input,
    };
    let additional_command_buffers = self.app.render(render_input);

    // End GUI frame, handle output, and render.
    self.gui_integration.end_frame_and_handle(&self.window, &self.gfx.device, &self.gfx.queue, self.screen_size, &gfx_frame.output_texture, &mut gfx_frame.encoder);

    // Submit command buffers
    let command_buffer = gfx_frame.encoder.finish();
    let command_buffers = std::iter::once(command_buffer).chain(additional_command_buffers);
    self.gfx.queue.submit(command_buffers);

    // Present
    surface_texture.present();

    return false; // Keep looping.
  }
}

// Frames

#[derive(Default)]
struct Frames {
  stopwatch: Stopwatch,
  frame: u64,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct FrameEnd {
  /// Duration of the frame. That is, the duration from the frame start to the frame end.
  pub duration: Offset,
}

impl Frames {
  fn start(&mut self, previous_frame_end: FrameEnd) -> Frame {
    let duration = previous_frame_end.duration;
    self.stopwatch.reset();
    Frame { frame: self.frame, duration }
  }

  fn end(&mut self) -> FrameEnd {
    let duration = self.stopwatch.elapsed();
    self.frame += 1;
    FrameEnd { duration }
  }
}

// Simulation updates

pub struct Updates {
  stopwatch: Stopwatch,
  target_duration: Offset,
  accumulated_lag: Offset,
  update: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct StepEnd {
  /// Duration of the step. That is, the duration from the step start to the step end.
  pub duration: Offset,
}

impl Updates {
  fn new(target_duration: Offset) -> Self {
    Self {
      stopwatch: Stopwatch::default(),
      target_duration,
      update: 0,
      accumulated_lag: Offset::default(),
    }
  }

  /// Returns the target duration: how much time a single update step should simulate. This is a fixed amount of time
  /// for determinism of update steps.
  pub fn target_duration(&self) -> Offset { self.target_duration }
  /// Returns the accumulated lag: the amount of time that the simulation lags behind the current frame.
  ///
  /// After performing as many simulation update steps as possible to minimize the accumulated lag, this will always
  /// return a number less than [target_duration](Self::target_duration), because no more steps could be performed.
  ///
  /// A renderer can render information from the simulation with extrapolation based on this lag.
  pub fn accumulated_lag(&self) -> Offset { self.accumulated_lag }
  /// Returns `true` when a step should be performed, `false` otherwise. A step should be performed when the accumulated
  /// lag equals or exceeds the target duration.
  pub fn should_step(&self) -> bool {
    self.accumulated_lag >= self.target_duration
  }
  /// Returns the ratio between accumulated lag and the target step duration. This ratio show how close we are to the
  /// next update step. A ratio of 0.0 means we have not accumulated any lag. A ratio of 1.0 means we have accumulated
  /// lag for exactly one update step. A ratio of 1.5 would mean we have accumulated lag for one and a half update
  /// steps, and so forth.
  ///
  /// After performing as many simulation update steps as possible to minimize the accumulated lag, this will always
  /// return a ration between 0.0 (inclusive) and 1.0 (exclusive), because no more steps could be performed.
  ///
  /// A renderer can render information from the simulation with extrapolation based on this ratio.
  pub fn target_ratio(&self) -> f64 {
    self.accumulated_lag / self.target_duration
  }

  /// Accumulate a lag of `duration`.
  pub fn accumulate_lag(&mut self, duration: Offset) -> Offset {
    self.accumulated_lag += duration;
    self.accumulated_lag
  }


  /// Starts a step and returns information about the step.
  pub fn start_step(&mut self) -> Step {
    self.stopwatch.reset();
    Step { update: self.update, target_duration: self.target_duration }
  }
  /// Ends the step, catching up on accumulated lag and returning information about the step.
  pub fn end_step(&mut self) -> StepEnd {
    self.accumulated_lag -= self.target_duration;
    let step_end = StepEnd { duration: self.stopwatch.elapsed() };
    self.update += 1;
    step_end
  }
}
