use thiserror::Error;
use winit::error::EventLoopError;
use winit::event_loop::{EventLoop, EventLoopBuilder};

pub struct OsContext {
  pub(crate) event_loop: EventLoop<()>,
}

#[derive(Debug, Error)]
#[error("Could not create OS context")]
pub struct OsContextCreateError(#[from] EventLoopError);

impl OsContext {
  pub fn new() -> Result<Self, OsContextCreateError> {
    let event_loop = EventLoopBuilder::new().build()?;
    let os_context = Self { event_loop };
    Ok(os_context)
  }
}
