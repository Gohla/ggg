#![feature(iter_collect_into)]

use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, never, Receiver, Sender, unbounded};
pub use crossbeam_channel::SendError;
use smallvec::SmallVec;

use manager::ManagerThread;
use worker::WorkerThread;

use crate::manager::FromQueueMessage;

mod worker;
mod manager;


// Message from manager

pub enum JobQueueMessage<J, I, O> {
  JobCompleted(J, Arc<O>),
  PendingJobRemoved(J, I),
  RunningJobRemoved(J),
  CompletedJobRemoved(J, Arc<O>),
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
  pub fn add_job(&self, job_key: J, input: I) -> Result<(), SendError<()>> {
    self.add_job_with_dependencies(job_key, Dependencies::default(), input)
  }

  #[inline]
  pub fn add_job_with_dependencies(&self, job_key: J, dependencies: Dependencies<J, D, DS>, input: I) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::AddJob(job_key, dependencies, input)).map_err(|_| SendError(()))
  }

  #[inline]
  pub fn remove_job_and_dependencies(&self, job_key: J) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::RemoveJobAndDependencies(job_key)).map_err(|_| SendError(()))
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
    let empty_receiver = never();
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
    let empty_receiver = never();
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

pub type DependencyOutputs<D, O, const DS: usize> = SmallVec<[(D, Arc<O>); DS]>;

pub trait Handler<J, D, I, O, const DS: usize>: Fn(J, DependencyOutputs<D, O, DS>, I) -> O + Send + 'static + Clone {}

impl<T, J, D, I, O, const DS: usize> Handler<J, D, I, O, DS> for T where T: Fn(J, DependencyOutputs<D, O, DS>, I) -> O + Send + 'static + Clone {}


// Trait aliases

pub trait JobKey: Send + 'static + Copy + Eq + Hash + Debug {}

impl<T> JobKey for T where T: Send + 'static + Copy + Eq + Hash + Debug {}


pub trait DepKey: Send + 'static + Copy {}

impl<T> DepKey for T where T: Send + 'static + Copy {}


pub trait In: Send + 'static {}

impl<T> In for T where T: Send + 'static {}


pub trait Out: Send + Sync + 'static {}

impl<T> Out for T where T: Send + Sync + 'static {}

