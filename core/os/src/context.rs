use thiserror::Error;
use winit::error::EventLoopError;
use winit::event_loop::EventLoop;

pub struct Context {
  pub(crate) event_loop: EventLoop<()>,
}

#[derive(Debug, Error)]
#[error("Could not create OS context: {0}")]
pub struct ContextCreateError(#[from] EventLoopError);

impl Context {
  pub fn new() -> Result<Self, ContextCreateError> {
    let event_loop = EventLoop::builder()
      .build()?;
    Ok(Self { event_loop })
  }
}
