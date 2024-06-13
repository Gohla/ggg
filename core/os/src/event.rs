use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::thread::JoinHandle;
use std::time::Duration;

use thiserror::Error;
use winit::application::ApplicationHandler;
pub use winit::error::EventLoopError;
use winit::event::{KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, DeviceEvents, EventLoopProxy};
pub use winit::event_loop::EventLoopClosed;
use winit::keyboard::{Key, ModifiersState, PhysicalKey, SmolStr};
use winit::window::WindowId;

use common::input::{KeyboardKey, KeyboardModifier, MouseButton, SemanticKey};
use common::line::LineDelta;
use common::screen::{PhysicalSize, Scale, ScreenDelta, ScreenPosition, ScreenSize};

use crate::window::{Window, WindowCreateError, WindowOptions};

#[derive(Clone, Debug)]
pub enum Event {
  WindowCursor { cursor_in_window: bool },
  WindowFocus { window_has_focus: bool },
  WindowSizeChange(ScreenSize),
  Stop,
}

#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub enum InputEvent {
  MouseButton { button: MouseButton, state: ElementState },
  MousePosition(ScreenPosition),
  /// Amount in pixels to scroll in the horizontal (x) and vertical (y) direction.
  ///
  /// Scroll events are expressed as a `MouseWheelPixelChange` if supported by the device (i.e., touchpad) and platform.
  ///
  /// Positive values indicate that the content being scrolled should move right/down.
  ///
  /// For a 'natural scrolling' touchpad (that acts like a touch screen) this means moving your fingers right and down
  /// should give positive values, and move the content right and down (to reveal more things left and up).
  MouseWheelPixel(ScreenDelta),
  /// Amount in lines or rows to scroll in the horizontal (x) and vertical (y) directions.
  ///
  /// Positive values indicate that the content that is being scrolled should move right and down (revealing more
  /// content left and up).
  MouseWheelLine(LineDelta),
  KeyboardModifier { modifier: KeyboardModifier, state: ElementState },
  KeyboardKey { keyboard_key: Option<KeyboardKey>, semantic_key: Option<SemanticKey>, text: Option<SmolStr>, state: ElementState },
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ElementState {
  Pressed,
  Released,
}
impl From<winit::event::ElementState> for ElementState {
  #[inline]
  fn from(element_state: winit::event::ElementState) -> Self {
    use winit::event::ElementState::*;
    match element_state {
      Pressed => Self::Pressed,
      Released => Self::Released,
    }
  }
}
impl From<bool> for ElementState {
  #[inline]
  fn from(pressed: bool) -> Self { if pressed { Self::Pressed } else { Self::Released } }
}
impl From<ElementState> for bool {
  fn from(element_state: ElementState) -> Self { element_state.is_pressed() }
}
impl ElementState {
  #[inline]
  pub fn is_pressed(self) -> bool { self == Self::Pressed }
  #[inline]
  pub fn is_released(self) -> bool { self == Self::Released }
}


// Create an event loop

pub struct EventLoop {
  event_loop: winit::event_loop::EventLoop<()>,
  runner: EventLoopRunner,
}

#[derive(Debug, Error)]
#[error("Failed to create event loop: {0}")]
pub struct EventLoopCreateError(#[from] EventLoopError);

pub type OnWindowCreatedFn = Box<dyn FnOnce(Window) -> Result<(), Box<dyn Error>> + 'static>;

impl EventLoop {
  pub fn new(window_options: WindowOptions) -> Result<(Self, Receiver<InputEvent>, Receiver<Event>), EventLoopCreateError> {
    let event_loop = winit::event_loop::EventLoop::builder()
      .build()?;

    let (input_event_tx, input_event_rx) = channel::<InputEvent>();
    let (event_tx, event_rx) = channel::<Event>();
    let runner = EventLoopRunner {
      input_event_tx,
      event_tx,

      window_options,
      wait_duration: Duration::from_secs(5),
      on_window_created: None,
      join_handle: None,

      window_data: None,
      keyboard_modifiers: ModifiersState::empty(),
    };

    let event_loop = Self { event_loop, runner };
    Ok((event_loop, input_event_rx, event_rx))
  }

  /// Create an [event loop stopper](EventLoopStopper) to stop the event loop from the outside (e.g., different thread).
  pub fn create_event_loop_stopper(&self) -> EventLoopStopper {
    EventLoopStopper(self.event_loop.create_proxy())
  }

  /// Set the `duration` to wait for after processing events.
  pub fn with_wait_duration(mut self, duration: Duration) -> Self {
    self.runner.wait_duration = duration;
    self
  }

  /// Sets the `join_handle` to periodically check and to wait for when the event loop stops.
  pub fn with_join_handle(mut self, join_handle: JoinHandle<()>) -> Self {
    self.runner.join_handle = Some(join_handle);
    self
  }

  /// Sets a callback that gets called once (on the event loop thread) when the event loop creates the window.
  pub fn with_on_window_created_callback(mut self, on_window_created: OnWindowCreatedFn) -> Self {
    self.runner.on_window_created = Some(on_window_created);
    self
  }
}


// Running the event loop

#[derive(Clone)]
pub struct EventLoopStopper(EventLoopProxy<()>);

#[derive(Debug, Error)]
#[error("Failed to run event loop: {0}")]
pub struct EventLoopRunError(#[from] EventLoopError);

impl EventLoop {
  /// Run the event loop on the current thread, blocking the current thread until the event loop is stopped.
  ///
  /// The event loop waits after processing events, but is woken up after some duration. This duration defaults to 5
  /// seconds, but can be changed with [`with_wait_duration`](EventLoop::with_wait_duration).
  ///
  /// The event loop stops when:
  /// - The window is closed.
  /// - [`EventLoopStopper::stop`] is called on a stopper returned from [`Self::create_event_loop_stopper`].
  /// - The event loop tries to send an input event when the `Receiver<InputEvent>` from [`EventLoop::new`] is dropped.
  /// - The event loop tries to send an event when the `Receiver<Event>` from [`EventLoop::new`]  is dropped.
  /// - The event loop is about to wait and the [`is_finished`](JoinHandle::is_finished) method of the
  ///   [join handle](Self::with_join_handle) (if set) returns `true`. Since the event loop is woken up periodically,
  ///   this check is performed periodically.
  ///
  /// When the event loop stops, if a [join handle](Self::with_join_handle) was set, it will wait for the corresponding
  /// thread to finish, and propagates any panics from that thread.
  #[cfg(not(target_arch = "wasm32"))]
  pub fn run(mut self) -> Result<(), EventLoopRunError> {
    self.event_loop.listen_device_events(DeviceEvents::Never);
    self.event_loop.run_app(&mut self.runner)?;
    Ok(())
  }

  // TODO: WASM code is out of date with rest of the code.
  #[cfg(target_arch = "wasm32")]
  pub fn run(mut self, os_context: Context, mut cycle: impl FnMut() -> bool + 'static) {
    os_context.event_loop.run(move |event, _, control_flow| {
      self.event_cycle(event, control_flow);
      if *control_flow == ControlFlow::Exit { // Close was requested in `event_loop`.
        return;
      }
      let stop = cycle();
      if stop {
        *control_flow = ControlFlow::Exit;
      } else {
        *control_flow = ControlFlow::Poll; // Event loop does everything, so run as fast as possible.
      }
    });
  }
}


// Stop the event loop

#[derive(Debug, Error)]
#[error("Failed to stop event loop: {0}")]
pub struct EventLoopStopError(#[from] EventLoopClosed<()>);

impl EventLoopStopper {
  /// Stop the event loop, returning an `Err` if the event loop has already stopped.
  pub fn stop(&self) -> Result<(), EventLoopStopError> {
    self.0.send_event(())?;
    Ok(())
  }
}


// Internals: event loop runner cycle

struct EventLoopRunner {
  input_event_tx: Sender<InputEvent>,
  event_tx: Sender<Event>,

  window_options: WindowOptions,
  wait_duration: Duration,
  on_window_created: Option<OnWindowCreatedFn>,
  join_handle: Option<JoinHandle<()>>,

  window_data: Option<WindowData>,
  keyboard_modifiers: ModifiersState,
}

struct WindowData {
  _window: Window,
  id: WindowId,
  inner_size: ScreenSize,
}
impl WindowData {
  fn new(window: Window) -> Self {
    let id = window.id();
    let inner_size = window.inner_size();
    Self { _window: window, id, inner_size }
  }

  #[inline]
  fn id_matches(&self, id: WindowId) -> bool { self.id == id }
  #[inline]
  fn scale_factor(&self) -> Scale { self.inner_size.scale }
  #[inline]
  fn physical_size(&self) -> PhysicalSize { self.inner_size.physical }

  #[inline]
  fn set_inner_size(&mut self, inner_size: ScreenSize) {
    self.inner_size = inner_size;
  }
}

impl ApplicationHandler for EventLoopRunner {
  fn resumed(&mut self, active_event_loop: &ActiveEventLoop) {
    match create_window(active_event_loop, &self.window_options, self.on_window_created.take()) {
      Ok(window) => self.window_data = Some(WindowData::new(window)),
      Err(cause) => {
        tracing::error!(?cause, "Failed to create window: {}; stopping event loop", cause);
        active_event_loop.exit();
      }
    }
  }

  fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
    event_loop.exit();
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    if let Some(window_data) = &mut self.window_data {
      if window_data.id_matches(window_id) {
        if let Ok(()) = handle_window_event(event_loop, &self.event_tx, &self.input_event_tx, window_data, &mut self.keyboard_modifiers, event) {
          event_loop.set_control_flow(ControlFlow::wait_duration(self.wait_duration))
        } else {
          // Exit the event loop if sending a message fails.
          event_loop.exit()
        }
      }
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if let Some(join_handle) = &self.join_handle {
      if join_handle.is_finished() {
        event_loop.exit();
      }
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    let join_handle = self.join_handle.take();
    if let Some(join_handle) = join_handle {
      // Wait for receiver thread to stop, ensuring our thread outlives the receiver thread.
      if let Err(e) = join_handle.join() {
        // Propagate panics from receiver thread to the current thread.
        std::panic::resume_unwind(e);
      }
    }
  }
}


#[derive(Error, Debug)]
enum CreateWindowError {
  #[error("Failed to create window: {0}")]
  WindowCreateFail(#[from] WindowCreateError),
  // #[error("Failed to get raw window handle: {0}")]
  // RawWindowHandleFail(#[from] HandleError),
  #[error("On window created callback failed: {0}")]
  OnWindowCreatedCallbackFail(#[from] Box<dyn Error>),
}

fn create_window(
  active_event_loop: &ActiveEventLoop,
  window_options: &WindowOptions,
  on_window_created: Option<OnWindowCreatedFn>,
) -> Result<Window, CreateWindowError> {
  let window = Window::new(active_event_loop, window_options)?;
  // let window_handle = window.as_winit_window().window_handle()?.as_raw();
  if let Some(on_window_created) = on_window_created {
    on_window_created(window.clone())?;
  }
  Ok(window)
}


/// Handle a window event.
///
/// Returns `Err(Exit)` if sending an event fails due to the receiver being dropped, indicating that the receiver is
/// stopping or has stopped.
#[profiling::function]
fn handle_window_event(
  event_loop: &ActiveEventLoop,
  event_tx: &Sender<Event>,
  input_event_tx: &Sender<InputEvent>,
  window_data: &mut WindowData,
  keyboard_modifiers: &mut ModifiersState,
  event: WindowEvent,
) -> Result<(), Stop> {
  match event {
    WindowEvent::Focused(focus) => {
      event_tx.send(Event::WindowFocus { window_has_focus: focus })?;
    }
    WindowEvent::MouseInput { state, button, .. } => {
      input_event_tx.send(InputEvent::MouseButton { button: button.into(), state: state.into() })?;
    }
    WindowEvent::CursorMoved { position, .. } => {
      let screen_position = ScreenPosition::from_physical_scale(position, window_data.inner_size.scale);
      input_event_tx.send(InputEvent::MousePosition(screen_position))?;
    }
    WindowEvent::CursorEntered { .. } => {
      event_tx.send(Event::WindowCursor { cursor_in_window: true })?;
    }
    WindowEvent::CursorLeft { .. } => {
      event_tx.send(Event::WindowCursor { cursor_in_window: false })?;
    }
    WindowEvent::MouseWheel { delta, .. } => {
      match delta {
        MouseScrollDelta::LineDelta(horizontal_delta_lines, vertical_delta_lines) =>
          input_event_tx.send(InputEvent::MouseWheelLine(LineDelta::from((horizontal_delta_lines, vertical_delta_lines))))?,
        MouseScrollDelta::PixelDelta(physical_position) => {
          let screen_delta = ScreenDelta::from_physical_scale(physical_position, window_data.inner_size.scale);
          input_event_tx.send(InputEvent::MouseWheelPixel(screen_delta))?;
        }
      };
    }
    WindowEvent::KeyboardInput { event: KeyEvent { physical_key, logical_key, text, state, .. }, .. } => {
      let keyboard_key = match physical_key {
        PhysicalKey::Code(key_code) => Some(key_code.into()),
        PhysicalKey::Unidentified(native_key_code) => {
          tracing::warn!("Received unidentified native key code '{:?}' as physical key; ignoring", native_key_code);
          None
        }
      };
      let semantic_key = match logical_key {
        Key::Named(named_key) => Some(named_key.into()),
        Key::Character(character_name) => {
          tracing::trace!("Received unnamed character '{:?}' as logical key; ignoring", character_name);
          None
        }
        Key::Unidentified(native_key) => {
          tracing::warn!("Received unidentified native key '{:?}' as logical key; ignoring", native_key);
          None
        }
        Key::Dead(o) => {
          tracing::warn!("Received dead key '{:?}' as logical key; ignoring", o);
          None
        }
      };
      let state = state.into();
      input_event_tx.send(InputEvent::KeyboardKey { keyboard_key, semantic_key, text, state })?;
    }
    WindowEvent::ModifiersChanged(modifiers) => {
      let pressed = modifiers.state() - *keyboard_modifiers;
      {
        let state = ElementState::Pressed;
        if pressed.contains(ModifiersState::SHIFT) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Shift, state })?;
        }
        if pressed.contains(ModifiersState::CONTROL) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Control, state })?;
        }
        if pressed.contains(ModifiersState::ALT) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Alternate, state })?;
        }
        if pressed.contains(ModifiersState::SUPER) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Super, state })?;
        }
      }

      let released = *keyboard_modifiers - modifiers.state();
      {
        let state = ElementState::Released;
        if released.contains(ModifiersState::SHIFT) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Shift, state })?;
        }
        if released.contains(ModifiersState::CONTROL) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Control, state })?;
        }
        if released.contains(ModifiersState::ALT) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Alternate, state })?;
        }
        if released.contains(ModifiersState::SUPER) {
          input_event_tx.send(InputEvent::KeyboardModifier { modifier: KeyboardModifier::Super, state })?;
        }
      }

      *keyboard_modifiers = modifiers.state();
    }
    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
      event_loop.exit();
      event_tx.send(Event::Stop)?;
    }
    WindowEvent::Resized(inner_size) => {
      let inner_size = ScreenSize::from_physical_scale(inner_size, window_data.scale_factor());
      window_data.set_inner_size(inner_size);
      event_tx.send(Event::WindowSizeChange(inner_size))?;
    }
    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
      let inner_size = ScreenSize::from_physical_scale(window_data.physical_size(), scale_factor);
      window_data.set_inner_size(inner_size);
      event_tx.send(Event::WindowSizeChange(inner_size))?;
    }
    _ => {}
  }
  Ok(())
}

// Internal error type for stopping the event loop.
struct Stop;
impl<T> From<SendError<T>> for Stop {
  #[inline]
  fn from(_value: SendError<T>) -> Self { Self }
}
