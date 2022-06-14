use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use flume::{Receiver, Sender};
use petgraph::prelude::*;
use petgraph::visit::Walker;
use rustc_hash::FxHashMap;
use tracing::trace;

use crate::{Dependencies, DependencyOutputs, DepKey, In, JobKey, JobQueueMessage, Out};

// Message from queue

pub(crate) enum FromQueueMessage<J, D, I, const DS: usize> {
  TryAddJob(J, Dependencies<J, D, DS>, I),
  TryRemoveJobAndOrphanedDependencies(J),
}

// Manager thread

pub(crate) type FromQueue<J, D, I, const DS: usize> = FromQueueMessage<J, D, I, DS>;
pub(crate) type FromWorker<J, O> = (J, O);

pub(super) struct ManagerThread<J, D, I, O, const DS: usize> {
  from_queue: Receiver<FromQueue<J, D, I, DS>>,
  to_worker: Sender<crate::worker::FromManager<J, D, I, O, DS>>,
  from_worker: Receiver<FromWorker<J, O>>,
  to_queue: Sender<JobQueueMessage<J, I, O>>,

  job_graph: DiGraphMap<J, D>,
  job_key_to_job_status: FxHashMap<J, JobStatus<I, O>>,
  pending_jobs: u32,
  running_jobs: u32,
}

impl<J: JobKey, D: DepKey, I: In, O: Out, const DS: usize> ManagerThread<J, D, I, O, DS> {
  #[inline]
  pub(super) fn new(
    from_queue: Receiver<FromQueue<J, D, I, DS>>,
    to_worker: Sender<crate::worker::FromManager<J, D, I, O, DS>>,
    from_worker: Receiver<FromWorker<J, O>>,
    to_queue: Sender<JobQueueMessage<J, I, O>>,
  ) -> Self {
    Self {
      from_queue,
      to_worker,
      from_worker,
      to_queue,
      job_graph: DiGraphMap::new(),
      job_key_to_job_status: FxHashMap::default(),
      pending_jobs: 0,
      running_jobs: 0,
    }
  }

  #[inline]
  pub(super) fn create_thread_and_run(self) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name("Job Queue Manager".into())
      .spawn(|| { self.run() })
  }

  #[inline]
  fn run(mut self) {
    profiling::register_thread!();
    trace!("Started job queue manager thread");
    let mut job_key_cache_1 = Vec::new();
    let mut job_key_cache_2 = Vec::new();
    loop {
      let r#continue = self.receive(&mut job_key_cache_1, &mut job_key_cache_2);
      if !r#continue { break; }
    }
    trace!("Stopped job queue manager thread");
  }

  #[inline]
  fn receive(&mut self, node_index_cache_1: &mut Vec<J>, node_index_cache_2: &mut Vec<J>) -> bool {
    let selected = flume::Selector::new()
      .recv(&self.from_queue, |message| {
        if let Ok(message) = message {
          Some(SelectedReceiver::FromQueue(message))
        } else {
          None // Job queue was dropped; stop this thread.
        }
      })
      .recv(&self.from_worker, |result| {
        if let Ok(result) = result {
          Some(SelectedReceiver::FromWorker(result))
        } else {
          None // All workers have disconnected; stop this thread.
        }
      })
      .wait();
    match selected {
      Some(SelectedReceiver::FromQueue(message)) => self.handle_from_queue(message, node_index_cache_1),
      Some(SelectedReceiver::FromWorker((job_key, output))) => self.handle_from_worker(job_key, output, node_index_cache_1, node_index_cache_2),
      None => false,
    }
  }


  #[inline]
  fn handle_from_queue(&mut self, message: FromQueueMessage<J, D, I, DS>, job_key_cache: &mut Vec<J>) -> bool {
    use FromQueueMessage::*;
    match message {
      TryAddJob(job_key, dependencies, input) => self.try_add_job(job_key, dependencies, input, job_key_cache),
      TryRemoveJobAndOrphanedDependencies(job_key) => self.try_remove_job_and_orphaned_dependencies(job_key, job_key_cache),
    }
  }
  
  #[inline]
  fn handle_from_worker(&mut self, job_key: J, output: O, job_key_cache_1: &mut Vec<J>, job_key_cache_2: &mut Vec<J>) -> bool {
    use JobStatus::*;
    match self.job_key_to_job_status.get(&job_key) {
      Some(Pending(_)) => return true, // Job was removed, added, and not scheduled while it was running -> don't complete it.
      Some(Completed(_)) => return true, // Job was removed, added, scheduled, and completed while it was running -> don't complete it.
      None => return true, // Job was removed while it was running -> don't complete it.
      _ => {}
    }
    // Complete job.
    if !self.complete_job(job_key, output) {
      return false;
    }
    // Try to schedule dependent jobs.
    job_key_cache_1.clear();
    self.job_graph.neighbors_directed(job_key, Incoming).collect_into(job_key_cache_1);
    for dependent_node_index in job_key_cache_1.drain(..) {
      if !self.try_schedule_job(dependent_node_index, job_key_cache_2) {
        return false;
      }
    }
    true
  }


  #[profiling::function]
  #[inline]
  fn try_add_job(&mut self, job_key: J, dependencies: Dependencies<J, D, DS>, input: I, job_key_cache: &mut Vec<J>) -> bool {
    if self.job_key_to_job_status.contains_key(&job_key) {
      return true;
    }
    self.job_graph.add_node(job_key);
    self.job_key_to_job_status.insert(job_key, JobStatus::Pending(input));
    self.pending_jobs += 1;
    trace!("Added job {:?}", job_key);
    // Add dependencies.
    for (dependency, dependency_job_key) in dependencies.into_iter() {
      if !self.job_graph.contains_node(dependency_job_key) {
        panic!("Attempt to add dependency to job {:?} which has not been added", dependency_job_key);
      }
      self.job_graph.add_edge(job_key, dependency_job_key, dependency);
      trace!("Added dependency from job {:?} to {:?}", job_key, dependency_job_key);
    }
    // Try to schedule job.
    self.try_schedule_job(job_key, job_key_cache)
  }

  #[profiling::function]
  #[inline]
  fn try_remove_job_and_orphaned_dependencies(&mut self, job_key: J, job_key_cache: &mut Vec<J>) -> bool {
    if !self.job_key_to_job_status.contains_key(&job_key) {
      return true;
    }
    trace!("Try to remove job {:?} along with orphaned dependencies", job_key);
    job_key_cache.clear();
    Bfs::new(&self.job_graph, job_key).iter(&self.job_graph).collect_into(job_key_cache);
    for j in job_key_cache.drain(..) {
      if self.job_graph.neighbors_directed(j, Incoming).next().is_some() {
        continue; // Job has incoming dependencies, can't remove it.
      }
      self.job_graph.remove_node(j);
      trace!("Removed job {:?}", j);
      let job_status = self.job_key_to_job_status.remove(&j).unwrap(); // Unwrap OK: mapping must exist.
      let send_success = match job_status {
        JobStatus::Pending(input) => {
          let send_success = self.to_queue.send(JobQueueMessage::PendingJobRemoved(j, input)).is_ok();
          let send_success = self.decrement_pending_jobs_and_send_queue_empty_if_applicable() | send_success;
          send_success
        }
        JobStatus::Running => {
          let send_success = self.to_queue.send(JobQueueMessage::RunningJobRemoved(j)).is_ok();
          let send_success = self.decrement_running_jobs_and_send_queue_empty_if_applicable() | send_success;
          send_success
        }
        JobStatus::Completed(output) => {
          let send_success = self.to_queue.send(JobQueueMessage::CompletedJobRemoved(j, output)).is_ok();
          send_success
        }
      };
      if !send_success { return false; }
    }
    true
  }


  #[profiling::function]
  #[inline]
  fn try_schedule_job(&mut self, job_key: J, job_key_cache: &mut Vec<J>) -> bool {
    trace!("Try to schedule job {:?}", job_key);
    if let JobStatus::Pending(_) = self.job_key_to_job_status[&job_key] {
      job_key_cache.clear();
      self.job_graph.neighbors_directed(job_key, Outgoing).collect_into(job_key_cache);
      if job_key_cache.iter().all(|j| self.job_key_to_job_status[j].is_completed()) {
        let mut dependency_outputs = DependencyOutputs::<D, O, DS>::new();
        for dependency_job_key in job_key_cache.drain(..) {
          let dependency_output = self.job_key_to_job_status[&dependency_job_key].clone_completed();
          let dependency_edge = self.job_graph[(job_key, dependency_job_key)];
          dependency_outputs.push((dependency_edge, dependency_output));
        }
        return self.schedule_job(job_key, dependency_outputs);
      }
    }
    true
  }

  #[inline]
  fn schedule_job(&mut self, job_key: J, dependencies: DependencyOutputs<D, O, DS>) -> bool {
    let job_status = self.job_key_to_job_status.get_mut(&job_key).unwrap(); // Unwrap OK: job must exist when schedule_job is called.
    if let JobStatus::Pending(input) = std::mem::replace(job_status, JobStatus::Running) {
      trace!("Scheduling job {:?}", job_key);
      self.pending_jobs -= 1;
      self.running_jobs += 1;
      return self.to_worker.send((job_key, dependencies, input)).is_ok();
    }
    true
  }

  #[inline]
  fn complete_job(&mut self, job_key: J, output: O) -> bool {
    trace!("Completing job {:?}", job_key);
    let wrapped = Arc::new(output);
    *self.job_key_to_job_status.get_mut(&job_key).unwrap() = JobStatus::Completed(wrapped.clone()); // Unwrap OK: job must exist when complete_job is called.
    if self.to_queue.send(JobQueueMessage::JobCompleted(job_key, wrapped)).is_err() { return false; }
    self.decrement_running_jobs_and_send_queue_empty_if_applicable()
  }


  #[inline]
  fn decrement_pending_jobs_and_send_queue_empty_if_applicable(&mut self) -> bool {
    if self.pending_jobs > 0 {
      self.pending_jobs -= 1;
      if self.pending_jobs == 0 && self.running_jobs == 0 {
        return self.to_queue.send(JobQueueMessage::QueueEmpty).is_ok();
      }
    } else {
      panic!("Attempt to decrement pending jobs while pending jobs is 0");
    }
    true
  }

  #[inline]
  fn decrement_running_jobs_and_send_queue_empty_if_applicable(&mut self) -> bool {
    if self.running_jobs > 0 {
      self.running_jobs -= 1;
      if self.running_jobs == 0 && self.pending_jobs == 0 {
        return self.to_queue.send(JobQueueMessage::QueueEmpty).is_ok();
      }
    } else {
      panic!("Attempt to decrement running jobs while running jobs is 0");
    }
    true
  }
}


// Job status

pub(super) enum JobStatus<I, O> {
  Pending(I),
  Running,
  Completed(Arc<O>),
}

impl<I, O> JobStatus<I, O> {
  #[inline]
  fn is_completed(&self) -> bool {
    match self {
      Self::Completed(_) => true,
      _ => false
    }
  }
  #[inline]
  fn clone_completed(&self) -> Arc<O> {
    match self {
      Self::Completed(arc) => arc.clone(),
      _ => panic!("Attempt to call `clone_completed` on non-`Completed` job status")
    }
  }
}

// Selected receiver

enum SelectedReceiver<J, D, I, O, const DS: usize> {
  FromQueue(FromQueue<J, D, I, DS>),
  FromWorker(FromWorker<J, O>),
}
