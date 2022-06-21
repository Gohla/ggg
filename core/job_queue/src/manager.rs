use std::thread;
use std::thread::JoinHandle;

use flume::{Receiver, Sender};
use petgraph::prelude::*;
use petgraph::visit::VisitMap;
use rustc_hash::{FxHashMap, FxHashSet};
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

  worker_thread_job_buffer_count: usize,

  job_graph: DiGraphMap<JK, DK>,
  job_key_to_job_status: FxHashMap<JK, JobStatus<I, O>>,
  jobs_to_add: FxHashMap<JK, J>,
  jobs_to_schedule: FxHashMap<JK, (DK, O)>,

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
    worker_thread_job_buffer_count: usize,
    dependency_output_cache_count: usize,
  ) -> Self {
    Self {
      from_queue,
      to_worker,
      from_worker,
      to_queue,

      worker_thread_job_buffer_count,

      job_graph: DiGraphMap::new(),
      job_key_to_job_status: FxHashMap::default(),
      jobs_to_add: FxHashMap::default(),
      jobs_to_schedule: FxHashMap::default(),

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
    let mut job_key_cache_2 = Vec::new();
    loop {
      let r#continue = self.receive(&mut job_key_cache_1, &mut job_key_cache_2);
      if !r#continue { break; }
    }
    trace!("Stopped job queue manager thread");
  }

  #[inline]
  fn receive(&mut self, job_key_cache_1: &mut Vec<JK>, job_key_cache_2: &mut Vec<JK>) -> bool {
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
      Some(SelectedReceiver::FromWorker((job_key, output, dependency_outputs))) => self.handle_from_worker(job_key, output, dependency_outputs, job_key_cache_1, job_key_cache_2),
      Some(SelectedReceiver::FromQueue(message)) => self.handle_from_queue(message, job_key_cache_1, job_key_cache_2),
      None => false,
    }
  }


  #[inline]
  fn handle_from_queue(&mut self, message: FromQueueMessage<JK, J>, job_key_cache_1: &mut Vec<JK>, job_key_cache_2: &mut Vec<JK>) -> bool {
    use FromQueueMessage::*;
    match message {
      TryAddJob(job) => self.try_add_job(job, job_key_cache_1, job_key_cache_2),
      TryRemoveJobAndOrphanedDependencies(job_key) => self.try_remove_job_and_orphaned_dependencies(job_key, job_key_cache_1),
    }
  }

  #[inline]
  fn handle_from_worker(&mut self, job_key: JK, output: O, mut dependency_outputs: Vec<(DK, O)>, job_key_cache_1: &mut Vec<JK>, job_key_cache_2: &mut Vec<JK>) -> bool {
    if self.dependency_output_cache.len() < self.dependency_output_cache.capacity() {
      dependency_outputs.clear();
      self.dependency_output_cache.push(dependency_outputs);
    }
    use JobStatus::*;
    match self.job_key_to_job_status.get(&job_key) {
      Some(Pending(_)) => return true, // Job was removed, added, and not scheduled while it was running -> don't complete it.
      Some(Completed(_)) => return true, // Job was removed, added, scheduled, and completed while it was running -> don't complete it.
      None => return true, // Job was removed while it was running -> don't complete it.
      _ => {} // Otherwise: continue.
    }
    // Complete job.
    if !self.complete_job(job_key, output) { return false; }
    // Try to schedule dependent jobs.
    job_key_cache_1.clear();
    self.job_graph.neighbors_directed(job_key, Incoming).collect_into(job_key_cache_1);
    for dependent_node_index in job_key_cache_1.drain(..) {
      if !self.try_schedule_job(dependent_node_index, job_key_cache_2) { return false; }
    }
    // Fill up `to_worker` to `worker_thread_job_buffer_count`.
    while self.to_worker.len() < self.worker_thread_job_buffer_count && !self.jobs_to_add.is_empty() {
      // OPTO: get rid of key copy and two unwraps; not possible with HashMap though?
      let top_job_key = *self.jobs_to_add.keys().next().unwrap(); // Unwrap OK: jobs_to_add is not empty.
      let job = self.jobs_to_add.remove(&top_job_key).unwrap(); // Unwrap OK: jobs_to_add contains that key.
      if !self.try_add_job(job, job_key_cache_1, job_key_cache_2) { return false; }
    }
    true
  }


  #[profiling::function]
  #[inline]
  fn try_add_job(&mut self, job: J, job_key_cache_1: &mut Vec<JK>, job_key_cache_2: &mut Vec<JK>) -> bool {
    let job_key = job.key();
    if self.job_key_to_job_status.contains_key(job_key) { return true; } // Job already exists in graph: done
    if self.jobs_to_add.contains_key(job_key) { return true; } // Job already exists in jobs to add map: done.
    if self.to_worker.len() >= self.worker_thread_job_buffer_count {
      // Number of jobs sent to workers is at level, don't add and schedule the job yet.
      self.jobs_to_add.insert(*job_key, job);
      return true;
    }
    job_key_cache_1.clear();
    self.force_add_job_and_dependencies(job, job_key_cache_1, None);
    for job_key in job_key_cache_1.drain(..) {
      // Try to schedule job.
      if !self.try_schedule_job(job_key, job_key_cache_2) { return false; }
    }
    true
  }

  #[inline(always)] // Inline always for tail call recursion?
  fn force_add_job_and_dependencies(&mut self, job: J, jobs_to_schedule: &mut Vec<JK>, depender_keys: Option<(DK, JK)>) {
    let job_key = job.key();
    if self.job_key_to_job_status.contains_key(job_key) {
      // Job already exists in graph; only add dependency if needed.
      if let Some((dependency_key, depender_job_key)) = depender_keys {
        self.add_dependency_edge(depender_job_key, *job_key, dependency_key);
      }
      return;
    }
    self.jobs_to_add.remove(job_key); // Remove from jobs_to_add, as we are force adding it.
    let job_key = *job_key;
    let (input, dependencies) = job.into();
    self.job_graph.add_node(job_key);
    self.job_key_to_job_status.insert(job_key, JobStatus::Pending(input));
    trace!("Added job {:?}", job_key);
    if let Some((dependency_key, depender_job_key)) = depender_keys {
      self.add_dependency_edge(depender_job_key, job_key, dependency_key);
    }
    self.pending_jobs += 1;
    jobs_to_schedule.push(job_key); // OPTO: try schedule jobs inline, as we have dependency information here?
    for (dependency_key, dependency_job) in dependencies {
      self.force_add_job_and_dependencies(dependency_job, jobs_to_schedule, Some((dependency_key, job_key))); // Tail call recursion?
    }
  }

  #[inline(always)]
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
  fn try_schedule_job(&mut self, job_key: JK, job_key_cache: &mut Vec<JK>) -> bool {
    trace!("Try to schedule job {:?}", job_key);
    if let JobStatus::Pending(_) = self.job_key_to_job_status[&job_key] {
      job_key_cache.clear();
      self.job_graph.neighbors_directed(job_key, Outgoing).collect_into(job_key_cache);
      if job_key_cache.iter().all(|j| self.job_key_to_job_status[j].is_completed()) {
        let mut dependency_outputs = self.dependency_output_cache.pop().unwrap_or_else(|| Vec::new());
        for dependency_job_key in job_key_cache.drain(..) {
          let dependency_output = self.job_key_to_job_status[&dependency_job_key].clone_output_completed();
          let dependency_edge = self.job_graph[(job_key, dependency_job_key)];
          dependency_outputs.push((dependency_edge, dependency_output));
        }
        return self.schedule_job(job_key, dependency_outputs);
      }
    }
    true
  }

  #[inline]
  fn schedule_job(&mut self, job_key: JK, dependencies: Vec<(DK, O)>) -> bool {
    let job_status = self.job_key_to_job_status.get_mut(&job_key).unwrap(); // Unwrap OK: job must exist when schedule_job is called.
    if let JobStatus::Pending(input) = std::mem::replace(job_status, JobStatus::Running) {
      trace!("Scheduling job {:?}", job_key);
      self.pending_jobs -= 1;
      self.running_jobs += 1;
      return self.to_worker.send((job_key, input, dependencies)).is_ok();
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
}


// Job status

pub(super) enum JobStatus<I, O> {
  Pending(I),
  Running,
  Completed(O),
}

impl<I, O: Out> JobStatus<I, O> {
  #[inline]
  fn is_completed(&self) -> bool {
    match self {
      Self::Completed(_) => true,
      _ => false
    }
  }
  #[inline]
  fn clone_output_completed(&self) -> O {
    match self {
      Self::Completed(output) => output.clone(),
      _ => panic!("Attempt to call `clone_output_completed` on non-`Completed` job status")
    }
  }
}

// Selected receiver

enum SelectedReceiver<JK, DK, J, O> {
  FromQueue(FromQueue<JK, J>),
  FromWorker(FromWorker<JK, DK, O>),
}
