use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use tracing::debug;
use winit::dpi::PhysicalPosition as WinitPhysicalPosition;
use winit::event::{ElementState as WinitElementState, Event, ModifiersState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ControlFlow;
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
  pub fn run(mut self, os_context: OsContext, app_thread_join_handle: JoinHandle<()>) -> ! {
    self.app_thread_join_handle = Some(app_thread_join_handle);
    os_context.event_loop.run(move |event, _, control_flow| {
      // Event loop does nothing else, so just wait until the next event. Set before `event_loop` as it can override it
      // to `ControlFlow::Exit`.
      *control_flow = ControlFlow::Wait;
      self.event_cycle(event, control_flow);
    });
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

  #[profiling::function]
  fn event_cycle(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
      Event::WindowEvent { event, window_id, .. } if window_id == self.window_id => {
        match event {
          WindowEvent::MouseInput { state, button, .. } => {
            let button = match button {
              WinitMouseButton::Left => MouseButton::Left,
              WinitMouseButton::Right => MouseButton::Right,
              WinitMouseButton::Middle => MouseButton::Middle,
              WinitMouseButton::Other(b) => MouseButton::Other(b),
            };
            self.input_event_tx.send(OsInputEvent::MouseInput { button, state: state.into() })
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::CursorMoved { position, .. } => {
            let screen_position = ScreenPosition::from_physical_scale(position.into_common(), self.inner_size.scale);
            self.input_event_tx.send(OsInputEvent::MouseMoved(screen_position))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::CursorEntered { .. } => {
            self.os_event_tx.send(OsEvent::MouseEnteredWindow)
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::CursorLeft { .. } => {
            self.os_event_tx.send(OsEvent::MouseLeftWindow)
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::MouseWheel { delta, .. } => {
            match delta {
              MouseScrollDelta::LineDelta(horizontal_delta_lines, vertical_delta_lines) =>
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedLines { horizontal_delta_lines: horizontal_delta_lines as f64, vertical_delta_lines: vertical_delta_lines as f64 })
                  .unwrap_or_else(|_| *control_flow = ControlFlow::Exit),
              MouseScrollDelta::PixelDelta(WinitPhysicalPosition { x, y }) => {
                let screen_delta = ScreenDelta::from_physical_scale((x as i64, y as i64), self.inner_size.scale);
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedPixels(screen_delta))
                  .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
              }
            };
          }
          WindowEvent::KeyboardInput { input, .. } => {
            if let Some(virtual_keycode) = input.virtual_keycode {
              let button: KeyboardButton = unsafe { std::mem::transmute(virtual_keycode) };
              let state = input.state.into();
              self.input_event_tx.send(OsInputEvent::KeyboardInput { button, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            } else {
              debug!("Received keyboard input event without virtual keycode: {:?}", input);
            }
          }
          WindowEvent::ModifiersChanged(modifiers) => {
            let pressed = modifiers - self.modifiers;
            let state = ElementState::Pressed;
            if pressed.contains(ModifiersState::SHIFT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::CTRL) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::ALT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::LOGO) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            let released = self.modifiers - modifiers;
            let state = ElementState::Released;
            if released.contains(ModifiersState::SHIFT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::CTRL) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::ALT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::LOGO) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            self.modifiers = modifiers;
          }
          WindowEvent::ReceivedCharacter(c) => {
            self.input_event_tx.send(OsInputEvent::CharacterInput(c))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::CloseRequested => {
            self.os_event_tx.send(OsEvent::TerminateRequested)
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            *control_flow = ControlFlow::Exit;
          }
          WindowEvent::Resized(inner_size) => {
            self.inner_size = ScreenSize::from_physical_scale(inner_size.into_common(), self.inner_size.scale);
            self.os_event_tx.send(OsEvent::WindowResized(self.inner_size))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            self.inner_size = ScreenSize::from_physical_scale(self.inner_size.physical, scale_factor);
            self.os_event_tx.send(OsEvent::WindowResized(self.inner_size))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          _ => {}
        }
      }
      Event::LoopDestroyed => {
        let join_handle = self.app_thread_join_handle.take();
        if let Some(join_handle) = join_handle {
          if let Err(e) = join_handle.join() {
            std::panic::resume_unwind(e);
          }
        }
      }
      _ => {}
    }
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
