use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use petgraph::graph::NodeIndex;
use tracing::trace;

use crate::{DependencyOutputs, DepKey, Handler, In, JobKey, Out};

pub(crate) type FromManager<J, D, I, O> = (NodeIndex, J, DependencyOutputs<D, O>, I);

pub(super) struct WorkerThread<J, D, O, I, H> {
  from_manager: Receiver<FromManager<J, D, I, O>>,
  to_manager: Sender<crate::manager::FromWorker<J, O>>,
  handler: H,
}

impl<J: JobKey, D: DepKey, I: In, O: Out, H: Handler<J, D, I, O>> WorkerThread<J, D, O, I, H> {
  #[inline]
  pub(super) fn new(
    from_manager: Receiver<FromManager<J, D, I, O>>,
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
    trace!("Started job queue worker thread {}", thread_index);
    loop {
      if let Ok((node_index, job_key, dependencies, input)) = self.from_manager.recv() {
        trace!("Running job {:?}", node_index);
        let output = (self.handler)(job_key, dependencies, input);
        if self.to_manager.send((node_index, job_key, output)).is_err() {
          break; // Manager has disconnected; stop this thread.
        }
      } else {
        break; // Manager has disconnected; stop this thread.
      }
    }
    trace!("Stopped job queue worker thread {}", thread_index);
  }
}
