use std::hash::BuildHasherDefault;
use std::thread;
use std::thread::JoinHandle;

use flume::{Receiver, Sender};
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use petgraph::prelude::*;
use petgraph::visit::VisitMap;
use rustc_hash::{FxHasher, FxHashMap, FxHashSet};
use tracing::trace;

use crate::{DepKey, In, Job, JobKey, JobQueueMessage, Out};

// Message from queue

pub(crate) enum FromQueueMessage<JK, J> {
  TryAddJob(J),
  TryRemoveJobAndOrphanedDependencies(JK),
}


// Manager thread

pub(crate) type FromQueue<JK, J> = FromQueueMessage<JK, J>;
pub(crate) type FromWorker<JK, DK, O> = (JK, O, Vec<(DK, O)>);

pub(super) struct ManagerThread<JK, DK, I, J, O> {
  from_queue: Receiver<FromQueue<JK, J>>,
  to_worker: Sender<crate::worker::FromManager<JK, DK, I, O>>,
  from_worker: Receiver<FromWorker<JK, DK, O>>,
  to_queue: Sender<JobQueueMessage<JK, I, O>>,

  target_running_job_count: usize,

  job_graph: DiGraphMap<JK, DK>,
  job_key_to_job_status: FxHashMap<JK, JobStatus<DK, I, O>>,
  jobs_to_add: LinkedHashMap<JK, J, BuildHasherDefault<FxHasher>>,
  jobs_to_run: LinkedHashSet<JK, BuildHasherDefault<FxHasher>>,

  dependency_output_cache: Vec<Vec<(DK, O)>>,
  bfs_cache: Bfs<JK, FxHashSet<JK>>,

  pending_jobs: u32,
  running_jobs: u32,
}

impl<JK: JobKey, DK: DepKey, I: In, J: Job<JK, DK, I>, O: Out> ManagerThread<JK, DK, I, J, O> {
  #[inline]
  pub(super) fn new(
    from_queue: Receiver<FromQueue<JK, J>>,
    to_worker: Sender<crate::worker::FromManager<JK, DK, I, O>>,
    from_worker: Receiver<FromWorker<JK, DK, O>>,
    to_queue: Sender<JobQueueMessage<JK, I, O>>,
    target_running_job_count: usize,
    dependency_output_cache_count: usize,
  ) -> Self {
    Self {
      from_queue,
      to_worker,
      from_worker,
      to_queue,

      target_running_job_count,

      job_graph: DiGraphMap::new(),
      job_key_to_job_status: FxHashMap::default(),
      jobs_to_add: LinkedHashMap::default(),
      jobs_to_run: LinkedHashSet::default(),

      dependency_output_cache: Vec::with_capacity(dependency_output_cache_count),
      bfs_cache: Bfs::default(),

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
    loop {
      let r#continue = self.receive(&mut job_key_cache_1);
      if !r#continue { break; }
    }
    trace!("Stopped job queue manager thread");
  }

  #[inline]
  fn receive(&mut self, job_key_cache_1: &mut Vec<JK>) -> bool {
    let selected = flume::Selector::new()
      .recv(&self.from_worker, |result| {
        if let Ok(result) = result {
          Some(SelectedReceiver::FromWorker(result))
        } else {
          None // All workers have disconnected; stop this thread.
        }
      })
      .recv(&self.from_queue, |message| {
        if let Ok(message) = message {
          Some(SelectedReceiver::FromQueue(message))
        } else {
          None // Job queue was dropped; stop this thread.
        }
      })
      .wait();
    match selected {
      Some(SelectedReceiver::FromWorker((job_key, output, dependency_outputs))) => self.handle_from_worker(job_key, output, dependency_outputs, job_key_cache_1),
      Some(SelectedReceiver::FromQueue(message)) => self.handle_from_queue(message, job_key_cache_1),
      None => false,
    }
  }


  #[inline]
  fn handle_from_queue(&mut self, message: FromQueueMessage<JK, J>, job_key_cache_1: &mut Vec<JK>) -> bool {
    use FromQueueMessage::*;
    match message {
      TryAddJob(job) => self.try_add_job(job),
      TryRemoveJobAndOrphanedDependencies(job_key) => self.try_remove_job_and_orphaned_dependencies(job_key, job_key_cache_1),
    }
  }

  #[inline]
  fn handle_from_worker(&mut self, job_key: JK, output: O, dependency_outputs: Vec<(DK, O)>, job_key_cache_1: &mut Vec<JK>) -> bool {
    self.reclaim_dependency_outputs(dependency_outputs);
    use JobStatus::*;
    match self.job_key_to_job_status.get(&job_key) {
      Some(Pending(_, _)) => return true, // Job was removed, added, and not scheduled while it was running -> don't complete it.
      Some(Completed(_)) => return true, // Job was removed, added, scheduled, and completed while it was running -> don't complete it.
      None => return true, // Job was removed while it was running -> don't complete it.
      _ => {} // Otherwise: continue.
    }
    // Try to make dependent jobs ready to run.
    job_key_cache_1.clear();
    self.job_graph.neighbors_directed(job_key, Incoming).collect_into(job_key_cache_1);
    for depender_job_key in job_key_cache_1.drain(..) {
      if !self.try_make_job_ready_to_run(depender_job_key, job_key, &output) { return false; }
    }
    // Complete job.
    if !self.complete_job(job_key, output) { return false; }
    // Run and add jobs up to the target.
    self.run_and_add_jobs_until_target()
  }


  #[profiling::function]
  #[inline]
  fn try_add_job(&mut self, job: J) -> bool {
    let job_key = job.key();
    if self.job_key_to_job_status.contains_key(job_key) { return true; } // Job already exists in graph: done
    if self.jobs_to_add.contains_key(job_key) { return true; } // Job already exists in jobs to add map: done.
    if self.to_worker.len() >= self.target_running_job_count {
      // Number of jobs sent to workers is at level, don't add and schedule the job yet.
      self.jobs_to_add.insert(*job_key, job);
      return true;
    }
    self.force_add_job_and_dependencies(job);
    self.run_jobs_until_target()
  }

  #[inline]
  fn force_add_job_and_dependencies(&mut self, job: J) -> Option<O> {
    let job_key = job.key();
    if let Some(job_status) = self.job_key_to_job_status.get(job_key) { // Job already exists.
      return job_status.clone_output_if_completed();
    }
    self.jobs_to_add.remove(job_key); // Remove from jobs_to_add, as we are force adding it.
    let job_key = *job_key;
    let (input, dependencies) = job.into();
    self.job_graph.add_node(job_key);
    trace!("Added job {:?}", job_key);
    let mut dependency_outputs = self.create_dependency_outputs();
    let mut can_run = true;
    for (dependency_key, dependency_job) in dependencies {
      let dependency_job_key = *dependency_job.key();
      let dependency_output = self.force_add_job_and_dependencies(dependency_job);
      self.add_dependency_edge(job_key, dependency_job_key, dependency_key);
      if let Some(dependency_output) = dependency_output {
        dependency_outputs.push((dependency_key, dependency_output));
      } else {
        can_run = false;
      }
    }
    self.job_key_to_job_status.insert(job_key, JobStatus::Pending(input, dependency_outputs));
    self.pending_jobs += 1;
    if can_run {
      self.jobs_to_run.insert(job_key);
    }
    None
  }

  #[inline]
  fn add_dependency_edge(&mut self, depender_job_key: JK, dependee_job_key: JK, dependency_key: DK) {
    self.job_graph.add_edge(depender_job_key, dependee_job_key, dependency_key);
    trace!("Added dependency {:?} from job {:?} to {:?}", dependency_key, depender_job_key, dependee_job_key);
  }


  #[profiling::function]
  #[inline]
  fn try_remove_job_and_orphaned_dependencies(&mut self, job_key: JK, job_key_cache: &mut Vec<JK>) -> bool {
    if !self.job_key_to_job_status.contains_key(&job_key) { return true; } // Job does not exist: done.
    if let Some(_) = self.jobs_to_add.remove(&job_key) {
      // Job was not added to the graph yet: done.
      trace!("Removed job {:?} which was not added to the dependency graph yet", job_key);
      return true;
    }
    trace!("Try to remove job {:?} along with orphaned dependencies", job_key);
    // OPTO: implement custom BFS that ignores entire subgraphs of nodes that have incoming dependencies.
    // Reset BFS traversal (using leaky API as it provides no API for this)
    self.bfs_cache.discovered.clear();
    self.bfs_cache.stack.clear();
    // Start BFS traversal (again using leaky API)
    self.bfs_cache.discovered.visit(job_key);
    self.bfs_cache.stack.push_front(job_key);
    // Run BFS traversal, putting items in `job_key_cache` so we can mutate the graph when we iterate that. Cannot use
    // Walker API as it takes ownership of `bfs_cache`.
    job_key_cache.clear();
    while let Some(j) = self.bfs_cache.next(&self.job_graph) {
      job_key_cache.push(j);
    }
    for j in job_key_cache.drain(..) {
      if self.job_graph.neighbors_directed(j, Incoming).next().is_some() {
        continue; // Job has incoming dependencies, can't remove it.
      }
      self.job_graph.remove_node(j);
      self.jobs_to_run.remove(&j);
      trace!("Removed job {:?}", j);
      let job_status = self.job_key_to_job_status.remove(&j).unwrap(); // Unwrap OK: mapping must exist.
      let send_success = match job_status {
        JobStatus::Pending(input, dependency_outputs) => {
          self.reclaim_dependency_outputs(dependency_outputs);
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
  fn try_make_job_ready_to_run(&mut self, depender_job_key: JK, dependee_job_key: JK, dependee_job_output: &O) -> bool {
    trace!("Try to make job {:?} ready to run due to completion of {:?}", depender_job_key, dependee_job_key);
    if let JobStatus::Pending(_, dependency_outputs) = self.job_key_to_job_status.get_mut(&depender_job_key).unwrap() { // Unwrap OK: job exists.
      let dependency_key = self.job_graph[(depender_job_key, dependee_job_key)];
      dependency_outputs.push((dependency_key, dependee_job_output.clone()));
      if self.job_graph.neighbors_directed(depender_job_key, Outgoing).count() == dependency_outputs.len() {
        self.jobs_to_run.insert(depender_job_key);
      }
    }
    true
  }

  #[inline]
  fn run_pending_job(&mut self, job_key: JK) -> bool {
    let job_status = self.job_key_to_job_status.get_mut(&job_key).unwrap(); // Unwrap OK: job must exist when `run_pending_job` is called.
    if let JobStatus::Pending(input, dependency_outputs) = std::mem::replace(job_status, JobStatus::Running) {
      trace!("Running job {:?}", job_key);
      self.pending_jobs -= 1;
      self.running_jobs += 1;
      if !self.to_worker.send((job_key, input, dependency_outputs)).is_ok() { return false; }
    }
    true
  }


  #[inline]
  fn run_jobs_until_target(&mut self) -> bool {
    while !self.jobs_to_run.is_empty() && self.to_worker.len() < self.target_running_job_count {
      let job_key = self.jobs_to_run.pop_front().unwrap(); // Unwrap OK: `jobs_to_run` is not empty.
      if !self.run_pending_job(job_key) { return false; }
    }
    true
  }

  #[inline]
  fn run_and_add_jobs_until_target(&mut self) -> bool {
    // First run jobs until target, to give jobs that are ready to run priority.
    if !self.run_jobs_until_target() { return false; }
    // Then add and run jobs until target.
    while !self.jobs_to_add.is_empty() && self.to_worker.len() < self.target_running_job_count {
      let (_, job) = self.jobs_to_add.pop_front().unwrap(); // Unwrap OK: `jobs_to_add` is not empty.
      self.force_add_job_and_dependencies(job);
      if !self.run_jobs_until_target() { return false; }
    }
    true
  }


  #[inline]
  fn complete_job(&mut self, job_key: JK, output: O) -> bool {
    trace!("Completing job {:?}", job_key);
    *self.job_key_to_job_status.get_mut(&job_key).unwrap() = JobStatus::Completed(output.clone()); // Unwrap OK: job must exist when complete_job is called.
    if self.to_queue.send(JobQueueMessage::JobCompleted(job_key, output)).is_err() { return false; }
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


  #[inline]
  fn reclaim_dependency_outputs(&mut self, mut dependency_outputs: Vec<(DK, O)>) {
    if self.dependency_output_cache.len() < self.dependency_output_cache.capacity() {
      dependency_outputs.clear();
      self.dependency_output_cache.push(dependency_outputs);
    }
  }

  #[inline]
  fn create_dependency_outputs(&mut self) -> Vec<(DK, O)> {
    self.dependency_output_cache.pop().unwrap_or_else(|| Vec::new())
  }
}


// Job status

pub(super) enum JobStatus<DK, I, O> {
  Pending(I, Vec<(DK, O)>),
  Running,
  Completed(O),
}

impl<DK, I, O: Out> JobStatus<DK, I, O> {
  #[inline]
  fn clone_output_if_completed(&self) -> Option<O> {
    match self {
      Self::Completed(output) => Some(output.clone()),
      _ => None,
    }
  }
}

// Selected receiver

enum SelectedReceiver<JK, DK, J, O> {
  FromQueue(FromQueue<JK, J>),
  FromWorker(FromWorker<JK, DK, O>),
}
