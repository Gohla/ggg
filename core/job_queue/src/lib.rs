#![feature(iter_collect_into)]

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::thread::{self, JoinHandle};

use flume::{bounded, Receiver, Sender, unbounded};
pub use flume::SendError;

use manager::ManagerThread;
use worker::WorkerThread;

use crate::manager::FromQueueMessage;

mod worker;
mod manager;


// Message from manager

pub enum JobQueueMessage<JK, I, O> {
  JobCompleted(JK, O),
  PendingJobRemoved(JK, I),
  RunningJobRemoved(JK),
  CompletedJobRemoved(JK, O),
  QueueEmpty,
}


// Job queue

pub struct JobQueue<JK, DK, I, J, O> {
  manager_thread_handle: Option<JoinHandle<()>>,
  worker_thread_handles: Vec<JoinHandle<()>>,
  to_manager: Sender<manager::FromQueue<JK, J>>,
  from_manager: Receiver<JobQueueMessage<JK, I, O>>,

  _dependency_key_phantom: PhantomData<DK>,
}

impl<JK: JobKey, DK: DepKey, I: In, J: Job<JK, DK, I>, O: Out> JobQueue<JK, DK, I, J, O> {
  pub fn new(
    worker_thread_count: usize,
    worker_thread_job_buffer_count: usize,
    dependency_output_cache_count: usize,
    handler: impl Handler<JK, DK, I, O>
  ) -> std::io::Result<Self> {
    assert!(worker_thread_count > 0, "Worker thread count must be higher than 0");
    assert!(worker_thread_job_buffer_count > 0, "Worker thread job buffer count must be higher than 0");

    let (external_to_manager_sender, external_to_manager_receiver) = unbounded();
    let (manager_to_worker_sender, manager_to_worker_receiver) = unbounded();
    let (worker_to_manager_sender, worker_to_manager_receiver) = unbounded();
    let (manager_to_external_sender, manager_to_external_receiver) = unbounded();

    let manager_thread = ManagerThread::new(
      external_to_manager_receiver,
      manager_to_worker_sender,
      worker_to_manager_receiver,
      manager_to_external_sender,
      worker_thread_job_buffer_count,
      dependency_output_cache_count,
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
      _dependency_key_phantom: PhantomData::default(),
    })
  }

  fn new_dummy() -> Self {
    let (empty_sender, _) = bounded(0);
    let (_, empty_receiver) = bounded(0);
    Self {
      manager_thread_handle: None,
      worker_thread_handles: Vec::new(),
      to_manager: empty_sender,
      from_manager: empty_receiver,
      _dependency_key_phantom: PhantomData::default(),
    }
  }


  #[inline]
  pub fn try_add_job(&self, job: J) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::TryAddJob(job)).map_err(|_| SendError(()))
  }

  #[inline]
  pub fn try_remove_job_and_orphaned_dependencies(&self, job_key: JK) -> Result<(), SendError<()>> {
    self.to_manager.send(FromQueueMessage::TryRemoveJobAndOrphanedDependencies(job_key)).map_err(|_| SendError(()))
  }


  #[inline]
  pub fn get_message_receiver(&self) -> &Receiver<JobQueueMessage<JK, I, O>> { &self.from_manager }


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

  /// Takes ownership of self by replacing it with a dummy job queue that does nothing, and then joins the taken self.
  pub fn take_and_join(&mut self) -> thread::Result<()> {
    let job_queue = std::mem::replace(self, Self::new_dummy());
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


// Job

pub trait Job<JK: JobKey, DK: DepKey, I: In>: Send + 'static {
  fn key(&self) -> &JK;

  type DependencyIterator: Iterator<Item=(DK, Self)>;
  fn into(self) -> (I, Self::DependencyIterator);
}


// Handler

// pub type DependencyOutputs<'a, DK, O> = ;

pub trait Handler<JK, DK, I, O>: FnMut(JK, I, &[(DK, O)]) -> O + Clone + Send + 'static {}

impl<T, JK, DK, I, O> Handler<JK, DK, I, O> for T where T: FnMut(JK, I, &[(DK, O)]) -> O + Clone + Send + 'static {}


// Trait aliases

pub trait JobKey: Send + Copy + Eq + Ord + Hash + Debug + 'static {}

impl<T> JobKey for T where T: Send + Copy + Eq + Ord + Hash + Debug + 'static {}


pub trait DepKey: Send + Copy + Debug + 'static {}

impl<T> DepKey for T where T: Send + Copy + Debug + 'static + {}


pub trait In: Send + 'static {}

impl<T> In for T where T: Send + 'static {}


pub trait Out: Send + Clone + 'static {}

impl<T> Out for T where T: Send + Clone + 'static {}

