#![feature(never_type)]

use egui::{Context, Ui};
use pollster::FutureExt as _;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wgpu::{Backends, CommandBuffer, Features, Limits, PowerPreference, PresentMode, TextureFormat};

use common::input::RawInput;
use common::screen::{LogicalSize, ScreenSize};
use common::timing::Duration;
use gfx::{Frame, Gfx};
use os::directory::Directories;
use os::window::OsWindow;

use crate::debug_gui::DebugGui;
use crate::run::CreateError;

mod run;
mod debug_gui;
mod config;

#[derive(Debug)]
pub struct Os {
  pub window: OsWindow,
  pub directories: Directories,
}

#[derive(Debug)]
pub struct Tick {
  pub time_target: Duration,
  pub count: u64,
}

pub struct GuiFrame {
  pub context: Context,
}

impl std::ops::Deref for GuiFrame {
  type Target = Context;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.context }
}

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

  pub depth_stencil_texture_format: Option<TextureFormat>,
  pub sample_count: u32,
}

impl Default for Options {
  fn default() -> Self {
    #[cfg(not(target_arch = "wasm32"))]
      let size = LogicalSize::new(1920.0, 1080.0);
    #[cfg(target_arch = "wasm32")]
      let size = os::window::get_browser_inner_size();
    #[allow(unused_mut)] let mut options = Options {
      name: "GGG application".to_string(),

      window_inner_size: size,
      window_min_inner_size: size,

      graphics_backends: Backends::all(),
      graphics_adapter_power_preference: PowerPreference::None,
      require_graphics_device_features: Features::empty(),
      request_graphics_device_features: Features::empty(),
      graphics_device_limits: default_limits(),
      graphics_swap_chain_present_mode: PresentMode::Immediate,

      depth_stencil_texture_format: Some(TextureFormat::Depth32Float),
      sample_count: 1,
    };
    #[cfg(target_os = "macos")] {
      options.graphics_swap_chain_present_mode = PresentMode::Mailbox;
    }
    options
  }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_limits() -> Limits { Limits::default() }

#[cfg(target_arch = "wasm32")]
fn default_limits() -> Limits { Limits::downlevel_webgl2_defaults() }

pub fn run_with_defaults<A: Application + 'static>(name: &str) -> Result<(), CreateError> {
  run::<A>(Options {
    name: name.into(),
    ..Options::default()
  })
}

pub fn run<A: Application + 'static>(options: Options) -> Result<(), CreateError> {
  #[cfg(feature = "profile-with-tracy")]
  tracy_client::Client::start();

  run::run::<A>(options).block_on()
}

