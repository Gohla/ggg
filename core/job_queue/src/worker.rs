use std::thread;
use std::thread::JoinHandle;

use flume::{Receiver, Sender};
use tracing::trace;

use crate::{DepKey, Handler, In, JobKey, Out};

pub(crate) type FromManager<JK, DK, I, O> = (JK, I, Vec<(DK, O)>);

pub(super) struct WorkerThread<JK, DK, I, O, H> {
  from_manager: Receiver<FromManager<JK, DK, I, O>>,
  to_manager: Sender<crate::manager::FromWorker<JK, DK, O>>,
  handler: H,
}

impl<JK: JobKey, DK: DepKey, I: In, O: Out, H: Handler<JK, DK, I, O>> WorkerThread<JK, DK, I, O, H> {
  #[inline]
  pub(super) fn new(
    from_manager: Receiver<FromManager<JK, DK, I, O>>,
    to_manager: Sender<crate::manager::FromWorker<JK, DK, O>>,
    handler: H,
  ) -> Self {
    Self {
      from_manager,
      to_manager,
      handler,
    }
  }

  #[inline]
  pub(super) fn create_thread_and_run(self, thread_index: usize) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name(format!("Job Queue Worker {}", thread_index))
      .spawn(move || { self.run(thread_index) })
  }

  #[inline]
  fn run(mut self, thread_index: usize) {
    profiling::register_thread!();
    trace!("Started job queue worker thread {}", thread_index);
    loop {
      if let Ok((job_key, input, dependency_outputs)) = self.from_manager.recv() {
        trace!("Running job {:?}", job_key);
        let output = (self.handler)(job_key, input, &dependency_outputs);
        if self.to_manager.send((job_key, output, dependency_outputs)).is_err() {
          break; // Manager has disconnected; stop this thread.
        }
      } else {
        break; // Manager has disconnected; stop this thread.
      }
    }
    trace!("Stopped job queue worker thread {}", thread_index);
  }
}
