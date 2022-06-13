use std::thread;
use std::thread::JoinHandle;

use flume::{Receiver, Sender};
use tracing::trace;

use crate::{DependencyOutputs, DepKey, Handler, In, JobKey, Out};

pub(crate) type FromManager<J, D, I, O, const DS: usize> = (J, DependencyOutputs<D, O, DS>, I);

pub(super) struct WorkerThread<J, D, O, I, const DS: usize, H> {
  from_manager: Receiver<FromManager<J, D, I, O, DS>>,
  to_manager: Sender<crate::manager::FromWorker<J, O>>,
  handler: H,
}

impl<J: JobKey, D: DepKey, I: In, O: Out, const DS: usize, H: Handler<J, D, I, O, DS>> WorkerThread<J, D, O, I, DS, H> {
  #[inline]
  pub(super) fn new(
    from_manager: Receiver<FromManager<J, D, I, O, DS>>,
    to_manager: Sender<crate::manager::FromWorker<J, O>>,
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
  fn run(self, thread_index: usize) {
    profiling::register_thread!();
    trace!("Started job queue worker thread {}", thread_index);
    loop {
      if let Ok((job_key, dependencies, input)) = self.from_manager.recv() {
        trace!("Running job {:?}", job_key);
        let output = (self.handler)(job_key, dependencies, input);
        if self.to_manager.send((job_key, output)).is_err() {
          break; // Manager has disconnected; stop this thread.
        }
      } else {
        break; // Manager has disconnected; stop this thread.
      }
    }
    trace!("Stopped job queue worker thread {}", thread_index);
  }
}
