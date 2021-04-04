use std::sync::mpsc::{channel, Receiver, Sender};

use winit::dpi::PhysicalPosition as WinitPhysicalPosition;
use winit::event::{ElementState as WinitElementState, Event, KeyboardInput, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::WindowId;

use common::screen::{PhysicalPosition, ScreenSize};

use crate::context::OsContext;
use crate::screen_ext::*;
use crate::window::OsWindow;

pub struct OsEventSys {
  input_event_tx: Sender<OsInputEvent>,
  os_event_tx: Sender<OsEvent>,
  window_id: WindowId,
  inner_size: ScreenSize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OsInputEvent {
  MouseInput { button: MouseButton, state: ElementState },
  MouseMoved(PhysicalPosition),
  // TODO: distinguish line and pixel delta.
  MouseWheelMoved { x_delta: f64, y_delta: f64 },
  // TODO: this contains a winit item, but it's pretty big to copy...
  KeyboardInput(KeyboardInput),
  CharacterInput(char),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OsEvent {
  TerminateRequested,
  WindowResized(ScreenSize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseButton {
  Left,
  Right,
  Middle,
  Other(u16),
}

impl From<WinitMouseButton> for MouseButton {
  fn from(mouse_button: WinitMouseButton) -> Self {
    match mouse_button {
      WinitMouseButton::Left => MouseButton::Left,
      WinitMouseButton::Right => MouseButton::Right,
      WinitMouseButton::Middle => MouseButton::Middle,
      WinitMouseButton::Other(b) => MouseButton::Other(b),
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ElementState {
  Pressed,
  Released,
}

impl From<WinitElementState> for ElementState {
  fn from(element_state: WinitElementState) -> Self {
    match element_state {
      WinitElementState::Pressed => ElementState::Pressed,
      WinitElementState::Released => ElementState::Released,
    }
  }
}


impl OsEventSys {
  pub fn new(window: &OsWindow) -> (OsEventSys, Receiver<OsInputEvent>, Receiver<OsEvent>) {
    let (input_event_tx, input_event_rx) = channel::<OsInputEvent>();
    let (os_event_tx, os_event_rx) = channel::<OsEvent>();
    let os_event_sys = OsEventSys {
      input_event_tx,
      os_event_tx,
      window_id: window.id(),
      inner_size: window.get_inner_size(),
    };
    (os_event_sys, input_event_rx, os_event_rx, )
  }

  pub fn run(mut self, os_context: OsContext) {
    os_context.event_loop.run(move |event, _, control_flow| {
      self.event_loop(event, control_flow);
    });
  }

  fn event_loop(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
      Event::WindowEvent { event, window_id, .. } if window_id == self.window_id => {
        match event {
          WindowEvent::MouseInput { state, button, .. } => {
            self.input_event_tx.send(OsInputEvent::MouseInput { button: button.into(), state: state.into() })
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::CursorMoved { position, .. } => {
            self.input_event_tx.send(OsInputEvent::MouseMoved(position.into_math()))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::MouseWheel { delta, .. } => {
            let (x_delta, y_delta) = match delta {
              MouseScrollDelta::LineDelta(x, y) => (x as f64, y as f64),
              MouseScrollDelta::PixelDelta(WinitPhysicalPosition { x, y }) => (x, y),
            };
            self.input_event_tx.send(OsInputEvent::MouseWheelMoved { x_delta, y_delta })
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
          }
          WindowEvent::KeyboardInput { input, .. } => {
            self.input_event_tx.send(OsInputEvent::KeyboardInput(input))
              .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
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
            self.inner_size = ScreenSize::from_physical_scale(inner_size.into_math(), self.inner_size.scale);
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
      _ => {}
    }
  }
}
