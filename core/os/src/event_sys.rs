use std::sync::mpsc::{channel, Receiver, Sender};

use tracing::debug;
use winit::dpi::PhysicalPosition as WinitPhysicalPosition;
use winit::event::{ElementState as WinitElementState, Event, ModifiersState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::WindowId;

use common::input::{KeyboardButton, KeyboardModifier, MouseButton};
use common::screen::{PhysicalPosition, ScreenSize};

use crate::context::OsContext;
use crate::screen_ext::*;
use crate::window::OsWindow;

pub struct OsEventSys {
  input_event_tx: Sender<OsInputEvent>,
  os_event_tx: Sender<OsEvent>,
  window_id: WindowId,
  modifiers: ModifiersState,
  inner_size: ScreenSize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OsInputEvent {
  MouseInput { button: MouseButton, state: ElementState },
  MouseMoved(PhysicalPosition),
  MouseWheelMovedPixels { horizontal_delta: f64, vertical_delta: f64 },
  MouseWheelMovedLines { horizontal_delta: f64, vertical_delta: f64 },
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
            self.input_event_tx.send(OsInputEvent::MouseMoved(position.into_math()))
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
              MouseScrollDelta::LineDelta(horizontal_delta, vertical_delta) =>
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedLines { horizontal_delta: horizontal_delta as f64, vertical_delta: vertical_delta as f64 })
                  .unwrap_or_else(|_| *control_flow = ControlFlow::Exit),
              MouseScrollDelta::PixelDelta(WinitPhysicalPosition { x: horizontal_delta, y: vertical_delta }) =>
                self.input_event_tx.send(OsInputEvent::MouseWheelMovedPixels { horizontal_delta, vertical_delta })
                  .unwrap_or_else(|_| *control_flow = ControlFlow::Exit),
            };
          }
          WindowEvent::KeyboardInput { input, .. } => {
            if let Some(virtual_keycode) = input.virtual_keycode {
              let button: common::input::KeyboardButton = unsafe { std::mem::transmute(virtual_keycode) };
              let state = input.state.into();
              self.input_event_tx.send(OsInputEvent::KeyboardInput { button, state })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            } else {
              debug!("Received keyboard input event without virtual keycode: {:?}", input);
            }
          }
          WindowEvent::ModifiersChanged(modifiers) => {
            let pressed = modifiers - self.modifiers;
            if pressed.contains(ModifiersState::SHIFT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state: ElementState::Pressed })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::CTRL) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state: ElementState::Pressed })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::ALT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state: ElementState::Pressed })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if pressed.contains(ModifiersState::LOGO) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state: ElementState::Pressed })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            let released = self.modifiers - modifiers;
            if released.contains(ModifiersState::SHIFT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Shift, state: ElementState::Released })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::CTRL) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Control, state: ElementState::Released })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::ALT) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Alternate, state: ElementState::Released })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
            if released.contains(ModifiersState::LOGO) {
              self.input_event_tx.send(OsInputEvent::KeyboardModifierChange { modifier: KeyboardModifier::Meta, state: ElementState::Released })
                .unwrap_or_else(|_| *control_flow = ControlFlow::Exit);
            }
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

impl From<WinitElementState> for ElementState {
  fn from(element_state: WinitElementState) -> Self {
    match element_state {
      WinitElementState::Pressed => ElementState::Pressed,
      WinitElementState::Released => ElementState::Released,
    }
  }
}
