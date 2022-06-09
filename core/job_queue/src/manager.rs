use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, select, Sender};
use petgraph::prelude::*;
use petgraph::visit::Walker;
use rustc_hash::FxHashMap;
use tracing::trace;

use crate::{Dependencies, DependencyOutputs, DepKey, In, JobKey, JobQueueMessage, Out};

// Message from queue

pub(crate) enum FromQueueMessage<J, D, I, const DS: usize> {
  AddJob(J, Dependencies<J, D, DS>, I),
  RemoveJobAndDependencies(J),
}

// Manager thread

pub(crate) type FromQueue<J, D, I, const DS: usize> = FromQueueMessage<J, D, I, DS>;
pub(crate) type FromWorker<J, O> = (NodeIndex, J, O);

pub(super) struct ManagerThread<J, D, I, O, const DS: usize> {
  from_queue: Receiver<FromQueue<J, D, I, DS>>,
  to_worker: Sender<crate::worker::FromManager<J, D, I, O, DS>>,
  from_worker: Receiver<FromWorker<J, O>>,
  to_queue: Sender<JobQueueMessage<J, I, O>>,

  job_graph: StableDiGraph<JobStatus<J, I, O>, D>,
  job_key_to_node_index: FxHashMap<J, NodeIndex>,
  pending_jobs: u32,
  running_jobs: u32,
}

impl<J: JobKey, D: DepKey, I: In, O: Out, const DS: usize> ManagerThread<J, D, I, O ,DS> {
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
      job_graph: StableDiGraph::new(),
      job_key_to_node_index: FxHashMap::default(),
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
    let mut node_index_cache_1 = Vec::new();
    let mut node_index_cache_2 = Vec::new();
    loop {
      select! {
        recv(self.from_queue) -> message => {
          if let Ok(message) = message {
            if !self.handle_message(message, &mut node_index_cache_2) {
              break; // Job queue or all workers have disconnected; stop this thread.
            }
          } else {
            break; // Job queue was dropped; stop this thread.
          }

        },
        recv(self.from_worker) -> result => {
          if let Ok((node_index, job_key, output)) = result {
            if !self.handle_completion(node_index, job_key, output, &mut node_index_cache_1, &mut node_index_cache_2) {
              break; // Job queue or all workers have disconnected; stop this thread.
            } 
          } else {
            break; // All workers have disconnected; stop this thread.
          }
        },
      }
    }
    trace!("Stopped job queue manager thread");
  }


  #[inline]
  fn handle_message(&mut self, message: FromQueueMessage<J, D, I, DS>, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    use FromQueueMessage::*;
    match message {
      AddJob(job_key, dependencies, input) => self.add_job(job_key, dependencies, input, node_index_cache),
      RemoveJobAndDependencies(job_key) => self.remove_job_and_dependencies(job_key, node_index_cache),
    }
  }

  #[profiling::function]
  #[inline]
  fn handle_completion(&mut self, node_index: NodeIndex, job_key: J, output: O, node_index_cache_1: &mut Vec<NodeIndex>, node_index_cache_2: &mut Vec<NodeIndex>) -> bool {
    use JobStatus::*;
    match self.job_graph.node_weight(node_index) {
      Some(Pending(_, _)) => return true, // Job was removed, added, and not scheduled while it was running -> don't complete it.
      Some(Completed(_, _)) => return true, // Job was removed, added, scheduled, and completed while it was running -> don't complete it.
      None => return true, // Job was removed while it was running -> don't complete it.
      _ => {}
    }
    if !self.job_graph.contains_node(node_index) {
      return true; // Job was removed, can't complete it and can't schedule dependent jobs.
    }
    // Complete job.
    if !self.complete_job(node_index, job_key, output) {
      return false;
    }
    // Try to schedule dependent jobs.
    node_index_cache_1.clear();
    self.job_graph.neighbors_directed(node_index, Incoming).collect_into(node_index_cache_1);
    for dependent_node_index in node_index_cache_1.drain(..) {
      if !self.try_schedule_job(dependent_node_index, node_index_cache_2) {
        return false;
      }
    }
    true
  }


  #[profiling::function]
  #[inline]
  fn add_job(&mut self, job_key: J, dependencies: Dependencies<J, D, DS>, input: I, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    if let Some(node_index) = self.job_key_to_node_index.get(&job_key) {
      panic!("Attempt to add job with key {:?} which already exists: {:?}", job_key, node_index);
      // Note: may allow this in the future by updating dependencies and re-executing the task if needed.
    } else {
      let node_index = self.job_graph.add_node(JobStatus::Pending(job_key, input));
      self.pending_jobs += 1;
      self.job_key_to_node_index.insert(job_key, node_index);
      trace!("Added job {:?} with key {:?}", node_index, job_key);
      // Add dependencies.
      for (dependency, dep_job_key) in dependencies.into_iter() {
        if let Some(dependency_node_index) = self.job_key_to_node_index.get(&dep_job_key) {
          let dependency_node_index = *dependency_node_index;
          self.job_graph.add_edge(node_index, dependency_node_index, dependency);
          trace!("Added dependency from job {:?} to {:?}", node_index, dependency_node_index);
        } else {
          panic!("Attempt to add dependency to job with key {:?} which has not been added", dep_job_key);
        }
      }
      // Try to schedule job.
      return self.try_schedule_job(node_index, node_index_cache);
    }
  }

  #[profiling::function]
  #[inline]
  fn remove_job_and_dependencies(&mut self, job_key: J, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    if let Some(node_index) = self.job_key_to_node_index.remove(&job_key) {
      trace!("Removing job {:?} with key {:?} along with dependencies", node_index, job_key);
      node_index_cache.clear();
      Dfs::new(&self.job_graph, node_index).iter(&self.job_graph).collect_into(node_index_cache);
      for n in node_index_cache.drain(..) {
        if self.job_graph.neighbors_directed(n, Incoming).next().is_some() {
          panic!("Attempt to remove job {:?} which has incoming dependencies", n);
        }
        if let Some(job_status) = self.job_graph.remove_node(n) {
          let (job_key, send_success) = match job_status {
            JobStatus::Pending(job_key, input) => {
              self.pending_jobs -= 1;
              let send_success = self.to_queue.send(JobQueueMessage::PendingJobRemoved(job_key, input)).is_ok();
              (job_key, send_success)
            }
            JobStatus::Running(job_key) => {
              self.running_jobs -= 1;
              let send_success = self.to_queue.send(JobQueueMessage::RunningJobRemoved(job_key)).is_ok();
              (job_key, send_success)
            }
            JobStatus::Completed(job_key, output) => {
              let send_success = self.to_queue.send(JobQueueMessage::CompletedJobRemoved(job_key, output)).is_ok();
              (job_key, send_success)
            }
          };
          if !send_success { return false; }
          self.job_key_to_node_index.remove(&job_key);
          if !self.send_queue_empty_if_applicable() { return false; }
        }
        trace!("Removed job {:?}", n);
      }
    }
    true
  }


  #[profiling::function]
  #[inline]
  fn try_schedule_job(&mut self, node_index: NodeIndex, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    trace!("Try to schedule job {:?}", node_index);
    if let Some(job_status) = self.job_graph.node_weight(node_index) {
      if let JobStatus::Pending(_, _) = job_status {
        node_index_cache.clear();
        self.job_graph.neighbors_directed(node_index, Outgoing).collect_into(node_index_cache);
        if node_index_cache.iter().all(|n| self.job_graph[*n].is_completed()) {
          let mut dependency_outputs = DependencyOutputs::<D, O, DS>::new();
          for dependency_node_index in node_index_cache.drain(..) {
            let dependency_output = self.job_graph[dependency_node_index].clone_completed();
            let dependency_edge_index = self.job_graph.find_edge(node_index, dependency_node_index).unwrap(); // Unwrap OK: edge exists.
            let dependency_edge = self.job_graph[dependency_edge_index];
            dependency_outputs.push((dependency_edge, dependency_output));
          }
          return self.schedule_job(node_index, dependency_outputs);
        }
      }
    }
    true
  }

  #[profiling::function]
  #[inline]
  fn schedule_job(&mut self, node_index: NodeIndex, dependencies: DependencyOutputs<D, O, DS>) -> bool {
    let job_status = &mut self.job_graph[node_index];
    trace!("Scheduling job {:?}", node_index);
    if let JobStatus::Pending(job_key, input) = std::mem::replace(job_status, JobStatus::Running(*job_status.get_job_key())) {
      self.pending_jobs -= 1;
      self.running_jobs += 1;
      return self.to_worker.send((node_index, job_key, dependencies, input)).is_ok();
    }
    true
  }

  #[profiling::function]
  #[inline]
  fn complete_job(&mut self, node_index: NodeIndex, job_key: J, output: O) -> bool {
    trace!("Completing job {:?}", node_index);
    let wrapped = Arc::new(output);
    self.job_graph[node_index] = JobStatus::Completed(job_key, wrapped.clone());
    self.running_jobs -= 1;
    if self.to_queue.send(JobQueueMessage::JobCompleted(job_key, wrapped)).is_err() { return false; }
    self.send_queue_empty_if_applicable()
  }


  #[profiling::function]
  #[inline]
  fn send_queue_empty_if_applicable(&mut self) -> bool {
    if self.pending_jobs == 0 && self.running_jobs == 0 {
      return self.to_queue.send(JobQueueMessage::QueueEmpty).is_ok();
    }
    true
  }
}


// Job status

pub(super) enum JobStatus<J, I, O> {
  Pending(J, I),
  Running(J),
  Completed(J, Arc<O>),
}

impl<J, I, O> JobStatus<J, I, O> {
  #[inline]
  fn get_job_key(&self) -> &J {
    match self {
      JobStatus::Pending(job_key, _) => job_key,
      JobStatus::Running(job_key) => job_key,
      JobStatus::Completed(job_key, _) => job_key,
    }
  }
  #[inline]
  fn is_completed(&self) -> bool {
    match self {
      Self::Completed(_, _) => true,
      _ => false
    }
  }
  #[inline]
  fn clone_completed(&self) -> Arc<O> {
    match self {
      Self::Completed(_, arc) => arc.clone(),
      _ => panic!("Attempt to call `clone_completed` on non-`Completed` job status")
    }
  }
}
