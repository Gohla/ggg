#![feature(iter_collect_into)]

use std::fmt::Debug;
use std::hash::Hash;
use std::thread::{self, JoinHandle};

use flume::{bounded, Receiver, Sender, unbounded};
pub use flume::SendError;
use smallvec::SmallVec;

use manager::ManagerThread;
use worker::WorkerThread;

use crate::manager::FromQueueMessage;

mod worker;
mod manager;


// Message from manager

pub enum JobQueueMessage<J, I, O> {
  JobCompleted(J, O),
  PendingJobRemoved(J, I),
  RunningJobRemoved(J),
  CompletedJobRemoved(J, O),
  QueueEmpty,
}


// Job queue

pub struct JobQueue<J, D, I, O, const DS: usize = 2> {
  manager_thread_handle: Option<JoinHandle<()>>,
  worker_thread_handles: Vec<JoinHandle<()>>,
  to_manager: Sender<manager::FromQueue<J, D, I, DS>>,
  from_manager: Receiver<JobQueueMessage<J, I, O>>,
}

impl<J: JobKey, D: DepKey, I: In, O: Out, const DS: usize> JobQueue<J, D, I, O, DS> {
  pub fn new(worker_thread_count: usize, handler: impl Handler<J, D, I, O, DS>) -> std::io::Result<Self> {
    let (external_to_manager_sender, external_to_manager_receiver) = unbounded();
    let (manager_to_worker_sender, manager_to_worker_receiver) = unbounded();
    let (worker_to_manager_sender, worker_to_manager_receiver) = unbounded();
    let (manager_to_external_sender, manager_to_external_receiver) = unbounded();

    let manager_thread = ManagerThread::new(
      external_to_manager_receiver,
      manager_to_worker_sender,
      worker_to_manager_receiver,
      manager_to_external_sender,
    );
    let manager_thread_handle = manager_thread.create_thread_and_run()?;

    let mut worker_thread_handles = Vec::new();
    for i in 0..worker_thread_count {
      let from_manager = manager_to_worker_receiver.clone();
      let to_manager = worker_to_manager_sender.clone();
      let handler = handler.clone();
      let worker_thread = WorkerThread::new(
        from_manager,
        to_manager,
        handler,
      );
      let worker_thread_handle = worker_thread.create_thread_and_run(i)?;
      worker_thread_handles.push(worker_thread_handle);
    }

    Ok(Self {
      manager_thread_handle: Some(manager_thread_handle),
      worker_thread_handles,
      to_manager: external_to_manager_sender,
      from_manager: manager_to_external_receiver,
    })
  }


  #[inline]
  pub fn try_add_job(&self, job_key: J, input: I) -> Result<(), SendError<()>> {
    self.try_add_job_with_dependencies(job_key, input, Dependencies::default())
  }

  #[inline]
  pub fn try_add_job_with_dependencies(&self, job_key: J, input: I, dependencies: Dependencies<J, D, DS>) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::TryAddJob(job_key, dependencies, input)).map_err(|_| SendError(()))
  }

  #[inline]
  pub fn try_remove_job_and_orphaned_dependencies(&self, job_key: J) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::TryRemoveJobAndOrphanedDependencies(job_key)).map_err(|_| SendError(()))
  }


  #[inline]
  pub fn get_message_receiver(&self) -> &Receiver<JobQueueMessage<J, I, O>> { &self.from_manager }


  pub fn stop_and_join(mut self) -> thread::Result<()> {
    self.stop();
    self.join()
  }

  pub fn stop(&mut self) {
    // Replace sender and receiver with new ones that do nothing, dropping the replaced ones.
    let (empty_sender, _) = bounded(0);
    drop(std::mem::replace(&mut self.to_manager, empty_sender));
    let (_, empty_receiver) = bounded(0);
    drop(std::mem::replace(&mut self.from_manager, empty_receiver));
  }

  /// Takes ownership of self by replacing it with a default job queue that does nothing, and then joins the taken self.
  pub fn take_and_join(&mut self) -> thread::Result<()> {
    let job_queue = std::mem::take(self);
    job_queue.join()
  }

  pub fn join(self) -> thread::Result<()> {
    if let Some(join_handle) = self.manager_thread_handle {
      join_handle.join()?;
    }
    for worker_thread in self.worker_thread_handles {
      worker_thread.join()?;
    }
    Ok(())
  }
}

impl<J, D, I, O, const DS: usize> Default for JobQueue<J, D, I, O, DS> {
  fn default() -> Self {
    let (empty_sender, _) = bounded(0);
    let (_, empty_receiver) = bounded(0);
    Self {
      manager_thread_handle: None,
      worker_thread_handles: Vec::new(),
      to_manager: empty_sender,
      from_manager: empty_receiver,
    }
  }
}


// Dependencies

pub type Dependencies<J, D, const DS: usize> = SmallVec<[(D, J); DS]>;


// Handler

pub type DependencyOutputs<D, O, const DS: usize> = SmallVec<[(D, O); DS]>;

pub trait Handler<J, D, I, O, const DS: usize>: FnMut(J, DependencyOutputs<D, O, DS>, I) -> O + Clone + Send + 'static {}

impl<T, J, D, I, O, const DS: usize> Handler<J, D, I, O, DS> for T where T: FnMut(J, DependencyOutputs<D, O, DS>, I) -> O + Clone + Send + 'static {}


// Trait aliases

pub trait JobKey: Send + Copy + Eq + Ord + Hash + Debug + 'static {}

impl<T> JobKey for T where T: Send + Copy + Eq + Ord + Hash + Debug + 'static {}


pub trait DepKey: Send + Copy + Debug + 'static {}

impl<T> DepKey for T where T: Send + Copy + Debug + 'static + {}


pub trait In: Send + 'static {}

impl<T> In for T where T: Send + 'static {}


pub trait Out: Send + Clone + 'static {}

impl<T> Out for T where T: Send + Clone + 'static {}

