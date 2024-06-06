#![feature(never_type)]

use egui::Ui;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wgpu::{Backends, CommandBuffer, Features, Limits, PowerPreference, PresentMode, TextureFormat};

use common::input::RawInput;
use common::screen::ScreenSize;
use common::timing::Duration;
use gfx::{Frame, Gfx};
use os::Os;
pub use run::RunError;

use crate::debug_gui::DebugGui;

mod run;
mod debug_gui;
mod config;

#[derive(Debug)]
pub struct Tick {
  pub time_target: Duration,
  pub count: u64,
}

pub struct GuiFrame {
  pub context: egui::Context,
}

impl std::ops::Deref for GuiFrame {
  type Target = egui::Context;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.context }
}


// Application trait

#[allow(unused_variables)]
pub trait Application: Sized {
  /// Type of configuration that is deserialized and passed into `new`, and gotten from `get_config` to be serialized.
  type Config: Default + Serialize + DeserializeOwned + Send + 'static;
  /// Creates an instance of the application with given `data`.
  fn new(os: &Os, gfx: &Gfx, config: Self::Config) -> Self;
  /// Takes the configuration of the application, so that the framework can serialize it.
  fn into_config(self) -> Self::Config { Self::Config::default() }

  /// Notifies the application about a screen resize or rescale event.
  fn screen_resize(&mut self, os: &Os, gfx: &Gfx, screen_size: ScreenSize) {}

  /// Return true to prevent the GUI from receiving keyboard events.
  fn is_capturing_keyboard(&self) -> bool { return false; }
  /// Return true to prevent the GUI from receiving mouse events.
  fn is_capturing_mouse(&self) -> bool { return false; }

  /// The type of input that this application creates, to be passed into `simulate` and `render`.
  type Input;
  /// Processes raw `input` into `Self::Input`.
  fn process_input(&mut self, input: RawInput) -> Self::Input;

  /// Allow the application to add elements to the debug menu via `ui`.
  fn add_to_debug_menu(&mut self, ui: &mut Ui) {}
  /// Allow the application to add elements to the menu bar via `ui`.
  fn add_to_menu(&mut self, ui: &mut Ui) {}

  /// Simulates a single `tick` of the application.
  fn simulate(&mut self, tick: Tick, input: &Self::Input) {}

  /// Renders a single `frame` of the application. May return additional command buffers to be submitted.
  fn render<'a>(&mut self, os: &Os, gfx: &Gfx, frame: Frame<'a>, gui_frame: &GuiFrame, input: &Self::Input) -> Box<dyn Iterator<Item=CommandBuffer>>;
}


// Options

pub struct Options {
  pub graphics_backends: Backends,
  pub graphics_adapter_power_preference: PowerPreference,

  pub require_graphics_device_features: Features,
  pub request_graphics_device_features: Features,
  pub graphics_device_limits: Limits,

  pub graphics_swap_chain_present_mode: PresentMode,

  pub depth_stencil_texture_format: Option<TextureFormat>,
  pub sample_count: u32,
}
impl Default for Options {
  fn default() -> Self {
    let graphics_device_limits = if cfg!(target_arch = "wasm32") {
      Limits::downlevel_webgl2_defaults()
    } else {
      Limits::default()
    };

    let graphics_swap_chain_present_mode = if cfg!(target_os = "macos") {
      PresentMode::Mailbox
    } else {
      PresentMode::Immediate
    };

    Self {
      graphics_backends: Backends::all(),
      graphics_adapter_power_preference: PowerPreference::None,

      require_graphics_device_features: Features::empty(),
      request_graphics_device_features: Features::empty(),
      graphics_device_limits,

      graphics_swap_chain_present_mode,

      depth_stencil_texture_format: Some(TextureFormat::Depth32Float),
      sample_count: 1,
    }
  }
}


// Application runner

#[derive(Default)]
pub struct AppRunner {
  os_options: os::Options,
  options: Options,
}

impl AppRunner {
  pub fn new(os_options: os::Options, options: Options) -> Self {
    Self { os_options, options }
  }
  pub fn from_name(name: impl Into<String>) -> Self {
    let os_options = os::Options {
      name: name.into(),
      ..os::Options::default()
    };
    Self {
      os_options,
      ..Self::default()
    }
  }
}

impl AppRunner {
  pub fn with_graphics_adapter_power_preference(mut self, power_preference: PowerPreference) -> Self {
    self.options.graphics_adapter_power_preference = power_preference;
    self
  }
  pub fn with_high_power_graphics_adapter(self) -> Self {
    self.with_graphics_adapter_power_preference(PowerPreference::HighPerformance)
  }
  pub fn with_low_power_graphics_adapter(self) -> Self {
    self.with_graphics_adapter_power_preference(PowerPreference::LowPower)
  }

  pub fn require_graphics_device_features(mut self, features: Features) -> Self {
    self.options.require_graphics_device_features.insert(features);
    self
  }
  pub fn request_graphics_device_features(mut self, features: Features) -> Self {
    self.options.request_graphics_device_features.insert(features);
    self
  }

  pub fn with_depth_stencil_texture_format(mut self, texture_format: TextureFormat) -> Self {
    self.options.depth_stencil_texture_format = Some(texture_format);
    self
  }
  pub fn without_depth_stencil_texture(mut self) -> Self {
    self.options.depth_stencil_texture_format = None;
    self
  }

  pub fn with_sample_count(mut self, sample_count: u32) -> Self {
    self.options.sample_count = sample_count;
    self
  }
}

impl AppRunner {
  pub fn run<A: Application>(self) -> Result<(), RunError> {
    use pollster::FutureExt;

    let (os, event_loop_runner) = Os::new(self.os_options)?;
    run::run::<A>(os, event_loop_runner, self.options).block_on()?;

    Ok(())
  }
}
