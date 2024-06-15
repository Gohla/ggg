use egui::Ui;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wgpu::{Backends, CommandBuffer, Features, Limits, PowerPreference, PresentMode, TextureFormat};

use common::input::RawInput;
use common::screen::ScreenSize;
use common::time::Offset;
use gfx::{Gfx, GfxFrame};
use gui::Gui;
use os::{ApplicationOptions, Os};
pub use run::RunError;

use crate::debug_gui::DebugGui;

mod run;
mod debug_gui;
mod config;

/// Frame information.
#[derive(Copy, Clone, Debug)]
pub struct Frame {
  /// Frame #
  pub frame: u64,
  /// Time elapsed since the previous frame. That is, the time elapsed from the previous frame start to the current
  /// frame start. Is set to `Offset::zero()` for the first frame.
  ///
  /// This can be used as an approximation for the duration of this frame.
  pub duration: Offset,
}

/// Simulation update step information.
#[derive(Copy, Clone, Debug)]
pub struct Step {
  /// Update #
  pub update: u64,
  /// How much time an update step should simulate. This is a fixed amount of time for determinism of update steps.
  pub target_duration: Offset,
}

/// Input for [Application::render]. Lifetime `'app` lives for the duration of the application, whereas, whereas
/// `'frame` only lives as long as a single frame.
pub struct RenderInput<'app, 'frame, A: Application> {
  /// Fully initialized operating system facade.
  pub os: &'app Os,
  /// Fully initialized graphics facade.
  pub gfx: &'app Gfx,
  /// Time elapsed since the start of the application.
  pub elapsed: Offset,

  /// Information about the current frame.
  pub frame: Frame,
  /// Application input for the current frame.
  pub input: &'frame A::Input,
  /// Handles and data for rendering a frame.
  pub gfx_frame: &'frame mut GfxFrame<'app>,
  /// Handles and data for creating GUIs.
  pub gui: Gui,
  /// The amount of time the simulated representation is behind the rendered representation. Extrapolation is required
  /// to make the rendered representation sync up with the simulated representation. Thus, the renderer should
  /// extrapolate this much time into the future.
  pub extrapolate: Offset,
}

/// Application trait
#[allow(unused_variables)]
pub trait Application: Sized {
  /// Type of configuration this application expects, passed into [new](Self::new) and retrieved with
  /// [into_config](Self::into_config).
  type Config: Default + Serialize + DeserializeOwned + Send + 'static;
  /// Create a new instance of the application.
  fn new(os: &Os, gfx: &Gfx, screen_size: ScreenSize, config: Self::Config) -> Self;
  /// Converts this application into its configuration.
  fn into_config(self) -> Self::Config { Self::Config::default() }

  /// Update this application with a new `screen_size`, possibly updating internal structures to reflect the new size.
  fn screen_resize(&mut self, os: &Os, gfx: &Gfx, screen_size: ScreenSize) {}

  /// Returns `true` when this application wants to capture keyboard input, `false` otherwise.
  fn wants_keyboard_input(&self) -> bool { return false; }
  /// Returns `true` when this application wants to capture mouse input, `false` otherwise.
  fn wants_mouse_input(&self) -> bool { return false; }

  /// Type of input this application creates, passed into `simulate` and `render`.
  type Input;
  /// Process raw `input` into [`Self::Input`].
  fn process_input(&mut self, input: RawInput) -> Self::Input;

  /// Add elements to the debug menu via `ui`.
  fn add_to_debug_menu(&mut self, ui: &mut Ui) {}
  /// Add elements to the menu bar via `ui`.
  fn add_to_menu(&mut self, ui: &mut Ui) {}

  /// Simulate a single update `step` with `input`.
  fn simulate(&mut self, step: Step, input: &Self::Input) {}

  /// Render a single frame from render `input`. May return additional command buffers to be submitted.
  fn render<'app, 'frame>(&mut self, input: RenderInput<'app, 'frame, Self>) -> Box<dyn Iterator<Item=CommandBuffer>>;
}

/// Application options
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
    let graphics_backends = if cfg!(target_os = "windows") {
      Backends::DX12
    } else {
      Backends::default()
    };
    let graphics_device_limits = if cfg!(target_arch = "wasm32") {
      Limits::downlevel_webgl2_defaults()
    } else {
      Limits::default()
    };

    Self {
      graphics_backends,
      graphics_adapter_power_preference: PowerPreference::None,

      require_graphics_device_features: Features::empty(),
      request_graphics_device_features: Features::empty(),
      graphics_device_limits,

      graphics_swap_chain_present_mode: PresentMode::AutoNoVsync,

      depth_stencil_texture_format: Some(TextureFormat::Depth32Float),
      sample_count: 1,
    }
  }
}

/// Application runner
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
      application: ApplicationOptions {
        name: name.into(),
        ..Default::default()
      },
      ..Default::default()
    };
    Self {
      os_options,
      ..Self::default()
    }
  }
}

impl AppRunner {
  pub fn with_graphics_backends(mut self, backends: Backends) -> Self {
    self.options.graphics_backends = backends;
    self
  }

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
    run::run_main_thread::<A>(self.os_options, self.options)
  }
}
