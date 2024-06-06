use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::thread::JoinHandle;

use thiserror::Error;
use tracing::debug;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition as WinitPhysicalPosition;
use winit::error::EventLoopError;
use winit::event::{KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::WindowId;

use common::input::{KeyboardButton, KeyboardModifier, MouseButton};
use common::screen::{ScreenDelta, ScreenPosition, ScreenSize};

use crate::context::Context;
use crate::window::Window;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum InputEvent {
  MouseInput { button: MouseButton, state: ElementState },
  MouseMoved(ScreenPosition),
  MouseWheelMovedPixels(ScreenDelta),
  MouseWheelMovedLines { horizontal_delta_lines: f64, vertical_delta_lines: f64 },
  KeyboardModifierChange { modifier: KeyboardModifier, state: ElementState },
  KeyboardInput { button: KeyboardButton, state: ElementState },
  CharacterInput(char),
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub enum Event {
  TerminateRequested,
  MouseEnteredWindow,
  MouseLeftWindow,
  WindowResized(ScreenSize),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ElementState {
  Pressed,
  Released,
}


// Create an event loop handler

pub struct EventLoopHandler {
  input_event_tx: Sender<InputEvent>,
  event_tx: Sender<Event>,

  window_id: WindowId,

  app_thread_join_handle: Option<JoinHandle<()>>,

  window_inner_size: ScreenSize,
  modifiers: ModifiersState,
}

impl EventLoopHandler {
  pub fn new(window: &Window) -> (Self, Receiver<InputEvent>, Receiver<Event>) {
    let (input_event_tx, input_event_rx) = channel::<InputEvent>();
    let (event_tx, event_rx) = channel::<Event>();
    let os_event_sys = Self {
      input_event_tx,
      event_tx,

      window_id: window.id(),

      app_thread_join_handle: None,

      modifiers: ModifiersState::empty(),
      window_inner_size: window.get_inner_size(),
    };
    (os_event_sys, input_event_rx, event_rx)
  }
}


// Create an event loop runner

pub struct EventLoopRunner {
  context: Context,
  event_handler: EventLoopHandler,
}
impl EventLoopRunner {
  pub fn new(context: Context, event_handler: EventLoopHandler) -> Self {
    Self { context, event_handler }
  }
}
impl EventLoopHandler {
  pub fn into_runner(self, context: Context) -> EventLoopRunner {
    EventLoopRunner::new(context, self)
  }
}


// Running the event loop

#[derive(Debug, Error)]
#[error("Failed to run event loop: {0}")]
pub struct EventLoopRunError(#[from] EventLoopError);

impl EventLoopRunner {
  /// Run the event loop on the current thread, blocking the current thread until the event loop is stopped.
  ///
  /// The event loop stops:
  /// - when the window is closed,
  /// - when the receiver end of the `Event` sender is dropped,
  /// - when `app_thread_join_handle.is_finished()` returns `true`.
  #[cfg(not(target_arch = "wasm32"))]
  pub fn run(mut self, app_thread_join_handle: JoinHandle<()>) -> Result<(), EventLoopRunError> {
    self.event_handler.app_thread_join_handle = Some(app_thread_join_handle);
    self.context.event_loop.run_app(&mut self.event_handler)?;
    Ok(())
  }

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


// Event loop cycle

impl ApplicationHandler for EventLoopHandler {
  fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    if window_id == self.window_id {
      if let Err(Exit) = self.handle_window_event(event_loop, event) {
        // Exit the event loop if sending a message fails.
        event_loop.exit()
      }
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if let Some(join_handle) = &self.app_thread_join_handle {
      // If the application thread has finished, also exit the event loop. This additional check is required because:
      // - Not all window events result in sending a message, thus the error in `window_event` would not be triggered.
      // - The application thread may have finished, but no window events are being raised, so `window_event` would not
      //   be called.
      if join_handle.is_finished() {
        event_loop.exit();
      }

      // TODO: `about_to_wait` may not even be called even when the application thread has finished. A user-event should
      //       be sent from the application thread to this event loop when the application thread finishes.
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    let join_handle = self.app_thread_join_handle.take();
    if let Some(join_handle) = join_handle {
      if let Err(e) = join_handle.join() {
        std::panic::resume_unwind(e);
      }
    }
  }
}

impl EventLoopHandler {
  /// Handle a window event.
  ///
  /// Returns `Err(Exit)` if sending a message to the application fails due to the receiver end being dropped,
  /// indicating that the application is exiting.
  #[profiling::function]
  fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) -> Result<(), Exit> {
    match event {
      WindowEvent::MouseInput { state, button, .. } => {
        self.input_event_tx.send(InputEvent::MouseInput { button: button.into(), state: state.into() })?;
      }
      WindowEvent::CursorMoved { position, .. } => {
        let screen_position = ScreenPosition::from_physical_scale(position, self.window_inner_size.scale);
        self.input_event_tx.send(InputEvent::MouseMoved(screen_position))?;
      }
      WindowEvent::CursorEntered { .. } => {
        self.event_tx.send(Event::MouseEnteredWindow)?;
      }
      WindowEvent::CursorLeft { .. } => {
        self.event_tx.send(Event::MouseLeftWindow)?;
      }
      WindowEvent::MouseWheel { delta, .. } => {
        match delta {
          MouseScrollDelta::LineDelta(horizontal_delta_lines, vertical_delta_lines) =>
            self.input_event_tx.send(InputEvent::MouseWheelMovedLines {
              horizontal_delta_lines: horizontal_delta_lines as f64,
              vertical_delta_lines: vertical_delta_lines as f64,
            })?,
          MouseScrollDelta::PixelDelta(WinitPhysicalPosition { x, y }) => {
            let screen_delta = ScreenDelta::from_physical_scale((x as i64, y as i64), self.window_inner_size.scale);
            self.input_event_tx.send(InputEvent::MouseWheelMovedPixels(screen_delta))?;
          }
        };
      }
      WindowEvent::KeyboardInput { event: KeyEvent { physical_key, text, state, .. }, .. } => {
        match physical_key {
          PhysicalKey::Code(key_code) => {
            self.input_event_tx.send(InputEvent::KeyboardInput { button: key_code.into(), state: state.into() })?;
          }
          PhysicalKey::Unidentified(native_key_code) => {
            debug!("Received unidentified native key code: {:?}", native_key_code);
          }
        }

        if let Some(text) = text {
          for c in text.chars() {
            self.input_event_tx.send(InputEvent::CharacterInput(c))?;
          }
        }
      }
      WindowEvent::ModifiersChanged(modifiers) => {
        let pressed = modifiers.state() - self.modifiers;
        {
          let state = ElementState::Pressed;
          if pressed.contains(ModifiersState::SHIFT) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })?;
          }
          if pressed.contains(ModifiersState::CONTROL) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })?;
          }
          if pressed.contains(ModifiersState::ALT) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })?;
          }
          if pressed.contains(ModifiersState::SUPER) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })?;
          }
        }

        let released = self.modifiers - modifiers.state();
        {
          let state = ElementState::Released;
          if released.contains(ModifiersState::SHIFT) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })?;
          }
          if released.contains(ModifiersState::CONTROL) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })?;
          }
          if released.contains(ModifiersState::ALT) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })?;
          }
          if released.contains(ModifiersState::SUPER) {
            self.input_event_tx.send(InputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })?;
          }
        }

        self.modifiers = modifiers.state();
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
        self.event_tx.send(Event::TerminateRequested)?;
      }
      WindowEvent::Resized(inner_size) => {
        self.window_inner_size = ScreenSize::from_physical_scale(inner_size, self.window_inner_size.scale);
        self.event_tx.send(Event::WindowResized(self.window_inner_size))?;
      }
      WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        self.window_inner_size = ScreenSize::from_physical_scale(self.window_inner_size.physical, scale_factor);
        self.event_tx.send(Event::WindowResized(self.window_inner_size))?;
      }
      _ => {}
    }
    Ok(())
  }
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

struct Exit;
impl<T> From<SendError<T>> for Exit {
  #[inline]
  fn from(_value: SendError<T>) -> Self { Self }
}