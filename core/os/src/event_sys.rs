use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::thread::JoinHandle;
use thiserror::Error;

use tracing::debug;
use winit::dpi::PhysicalPosition as WinitPhysicalPosition;
use winit::error::{EventLoopError};
use winit::event::{ElementState as WinitElementState, Event, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::WindowId;

use common::input::{KeyboardButton, KeyboardModifier, MouseButton};
use common::screen::{ScreenDelta, ScreenPosition, ScreenSize};

use crate::context::OsContext;
use crate::screen_ext::*;
use crate::window::OsWindow;

pub struct OsEventSys {
  input_event_tx: Sender<OsInputEvent>,
  os_event_tx: Sender<OsEvent>,
  window_id: WindowId,
  modifiers: ModifiersState,
  inner_size: ScreenSize,
  app_thread_join_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OsInputEvent {
  MouseInput { button: MouseButton, state: ElementState },
  MouseMoved(ScreenPosition),
  MouseWheelMovedPixels(ScreenDelta),
  MouseWheelMovedLines { horizontal_delta_lines: f64, vertical_delta_lines: f64 },
  KeyboardModifierChange { modifier: KeyboardModifier, state: ElementState },
  KeyboardInput { button: KeyboardButton, state: ElementState },
  CharacterInput(char),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OsEvent {
  TerminateRequested,
  MouseEnteredWindow,
  MouseLeftWindow,
  WindowResized(ScreenSize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ElementState {
  Pressed,
  Released,
}

#[derive(Debug, Error)]
#[error("Could not create window")]
pub struct EventSysRunError(#[from] EventLoopError);

impl OsEventSys {
  pub fn new(window: &OsWindow) -> (OsEventSys, Receiver<OsInputEvent>, Receiver<OsEvent>) {
    let (input_event_tx, input_event_rx) = channel::<OsInputEvent>();
    let (os_event_tx, os_event_rx) = channel::<OsEvent>();
    let os_event_sys = OsEventSys {
      input_event_tx,
      os_event_tx,
      window_id: window.id(),
      modifiers: ModifiersState::empty(),
      inner_size: window.get_inner_size(),
      app_thread_join_handle: None,
    };
    (os_event_sys, input_event_rx, os_event_rx, )
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn run(mut self, os_context: OsContext, app_thread_join_handle: JoinHandle<()>) -> Result<(), EventSysRunError> {
    self.app_thread_join_handle = Some(app_thread_join_handle);
    os_context.event_loop.run(move |event, target| {
      // Event loop does nothing else, so just wait until the next event. Set before `event_loop` as it can override it
      // to `ControlFlow::Exit`.
      target.set_control_flow(ControlFlow::Wait);
      self.event_cycle_catch(event, target);
    })?;
    Ok(())
  }

  #[cfg(target_arch = "wasm32")]
  pub fn run(mut self, os_context: OsContext, mut cycle: impl FnMut() -> bool + 'static) {
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

  fn event_cycle_catch(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) {
    if let Err(Exit) = self.event_cycle(event, target) {
      // Receiver end of `input_event_tx` hung up, indicating that the application is closing down.
      target.exit()
    }
  }

  #[profiling::function]
  fn event_cycle(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) -> Result<(), Exit> {
    match event {
      Event::WindowEvent { event, window_id, .. } if window_id == self.window_id => {
        match event {
          WindowEvent::MouseInput { state, button, .. } => {
            let button = match button {
              WinitMouseButton::Left => MouseButton::Left,
              WinitMouseButton::Right => MouseButton::Right,
              WinitMouseButton::Middle => MouseButton::Middle,
              WinitMouseButton::Back => MouseButton::Back,
              WinitMouseButton::Forward => MouseButton::Forward,
              WinitMouseButton::Other(b) => MouseButton::Other(b),
            };
            self.input_event_tx.send(OsInputEvent::MouseInput { button, state: state.into() })?;
          }
          WindowEvent::CursorMoved { position, .. } => {
            let screen_position = ScreenPosition::from_physical_scale(position.into_common(), self.inner_size.scale);
            self.input_event_tx.send(OsInputEvent::MouseMoved(screen_position))?;
          }
          WindowEvent::CursorEntered { .. } => {
            self.os_event_tx.send(OsEvent::MouseEnteredWindow)?;
          }
          WindowEvent::CursorLeft { .. } => {
            self.os_event_tx.send(OsEvent::MouseLeftWindow)?;
          }
          WindowEvent::MouseWheel { delta, .. } => {
            match delta {
              MouseScrollDelta::LineDelta(horizontal_delta_lines, vertical_delta_lines) =>
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedLines {
                  horizontal_delta_lines: horizontal_delta_lines as f64,
                  vertical_delta_lines: vertical_delta_lines as f64,
                })?,
              MouseScrollDelta::PixelDelta(WinitPhysicalPosition { x, y }) => {
                let screen_delta = ScreenDelta::from_physical_scale((x as i64, y as i64), self.inner_size.scale);
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedPixels(screen_delta))?;
              }
            };
          }
          WindowEvent::KeyboardInput { event: KeyEvent { physical_key, text, state, .. }, .. } => {
            match physical_key {
              PhysicalKey::Code(key_code) => {
                let button: KeyboardButton = unsafe { std::mem::transmute(key_code) };
                let state = state.into();
                self.input_event_tx.send(OsInputEvent::KeyboardInput { button, state })?;
              }
              PhysicalKey::Unidentified(native_key_code) => {
                debug!("Received unidentified native key code: {:?}", native_key_code);
              }
            }

            if let Some(text) = text {
              for c in text.chars() {
                self.input_event_tx.send(OsInputEvent::CharacterInput(c))?;
              }
            }
          }
          WindowEvent::ModifiersChanged(modifiers) => {
            let pressed = modifiers.state() - self.modifiers;
            {
              let state = ElementState::Pressed;
              if pressed.contains(ModifiersState::SHIFT) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })?;
              }
              if pressed.contains(ModifiersState::CONTROL) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })?;
              }
              if pressed.contains(ModifiersState::ALT) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })?;
              }
              if pressed.contains(ModifiersState::SUPER) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })?;
              }
            }

            let released = self.modifiers - modifiers.state();
            {
              let state = ElementState::Released;
              if released.contains(ModifiersState::SHIFT) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })?;
              }
              if released.contains(ModifiersState::CONTROL) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })?;
              }
              if released.contains(ModifiersState::ALT) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })?;
              }
              if released.contains(ModifiersState::SUPER) {
                self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })?;
              }
            }

            self.modifiers = modifiers.state();
          }
          WindowEvent::CloseRequested => {
            self.os_event_tx.send(OsEvent::TerminateRequested)?;
            target.exit();
          }
          WindowEvent::Resized(inner_size) => {
            self.inner_size = ScreenSize::from_physical_scale(inner_size.into_common(), self.inner_size.scale);
            self.os_event_tx.send(OsEvent::WindowResized(self.inner_size))?;
          }
          WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            self.inner_size = ScreenSize::from_physical_scale(self.inner_size.physical, scale_factor);
            self.os_event_tx.send(OsEvent::WindowResized(self.inner_size))?;
          }
          _ => {}
        }
      }
      Event::LoopExiting => {
        let join_handle = self.app_thread_join_handle.take();
        if let Some(join_handle) = join_handle {
          if let Err(e) = join_handle.join() {
            std::panic::resume_unwind(e);
          }
        }
      }
      _ => {}
    }

    Ok(())
  }
}

impl From<WinitElementState> for ElementState {
  fn from(element_state: WinitElementState) -> Self {
    match element_state {
      WinitElementState::Pressed => ElementState::Pressed,
      WinitElementState::Released => ElementState::Released,
    }
  }
}

struct Exit;
impl<T> From<SendError<T>> for Exit {
  #[inline]
  fn from(_value: SendError<T>) -> Self { Self }
}
