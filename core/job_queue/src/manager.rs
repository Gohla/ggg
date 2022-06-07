use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, select, Sender};
use petgraph::prelude::*;
use petgraph::visit::Walker;
use rustc_hash::FxHashMap;
use tracing::trace;

use crate::{Dependencies, DependencyOutputs, DepKey, FromManagerMessage, JobKey, Out};

// Message from queue

pub(crate) enum FromQueueMessage<J, D> {
  AddJob(J, Dependencies<J, D>),
  RemoveJobAndDependencies(J),
}

// Manager thread

pub(crate) type FromQueue<J, D> = FromQueueMessage<J, D>;
pub(crate) type FromWorker<J, O> = (NodeIndex, J, O);

pub(super) struct ManagerThread<J, D, O> {
  from_queue: Receiver<FromQueue<J, D>>,
  to_worker: Sender<crate::worker::FromManager<J, D, O>>,
  from_worker: Receiver<FromWorker<J, O>>,
  to_queue: Sender<FromManagerMessage<J, O>>,

  job_graph: StableDiGraph<JobStatus<J, O>, D>,
  job_key_to_node_index: FxHashMap<J, NodeIndex>,
  pending_jobs: u32,
  scheduled_jobs: u32,
}

impl<J: JobKey, D: DepKey, O: Out> ManagerThread<J, D, O> {
  #[inline]
  pub(super) fn new(
    from_queue: Receiver<FromQueue<J, D>>,
    to_worker: Sender<crate::worker::FromManager<J, D, O>>,
    from_worker: Receiver<FromWorker<J, O>>,
    to_queue: Sender<FromManagerMessage<J, O>>,
  ) -> Self {
    Self {
      from_queue,
      to_worker,
      from_worker,
      to_queue,
      job_graph: StableDiGraph::new(),
      job_key_to_node_index: FxHashMap::default(),
      pending_jobs: 0,
      scheduled_jobs: 0,
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
    trace!("Started job queue manager thread");
    let mut node_index_cache_1 = Vec::new();
    let mut node_index_cache_2 = Vec::new();
    loop {
      select! {
        recv(self.from_queue) -> message => {
          let Ok(message) = message else {
             break; // Job queue was dropped; stop this thread.
          };
          if !self.handle_message(message, &mut node_index_cache_2) {
            break; // All workers have disconnected; stop this thread.
          }
        },
        recv(self.from_worker) -> result => {
          let Ok((node_index, job_key, output)) = result else {
            break; // All workers have disconnected; stop this thread.
          };
          if !self.handle_completion(node_index, job_key, output, &mut node_index_cache_1, &mut node_index_cache_2) {
            break; // All workers have disconnected; stop this thread.
          }
        },
      }
    }
    trace!("Stopped job queue manager thread");
  }


  #[inline]
  fn handle_message(&mut self, message: FromQueueMessage<J, D>, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    use FromQueueMessage::*;
    match message {
      AddJob(job_key, dependencies) => self.add_job(job_key, dependencies, node_index_cache),
      RemoveJobAndDependencies(job_key) => self.remove_job_and_dependencies(job_key, node_index_cache),
    }
  }

  #[inline]
  fn handle_completion(&mut self, node_index: NodeIndex, job_key: J, output: O, node_index_cache_1: &mut Vec<NodeIndex>, node_index_cache_2: &mut Vec<NodeIndex>) -> bool {
    // Check if job is still in the dependency graph; it could have been removed while running.
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


  #[inline]
  fn add_job(&mut self, job_key: J, dependencies: Dependencies<J, D>, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    if let Some(node_index) = self.job_key_to_node_index.get(&job_key) {
      panic!("Attempt to add job with key {:?} which already exists: {:?}", job_key, node_index);
      // Note: may allow this in the future by updating dependencies and re-executing the task if needed.
    } else {
      let node_index = self.job_graph.add_node(JobStatus::Pending(job_key));
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
          match job_status {
            JobStatus::Pending(_) => self.pending_jobs -= 1,
            JobStatus::Running => self.scheduled_jobs -= 1,
            _ => {}
          }
          if !self.send_queue_empty_if_applicable() {
            return false;
          }
        }
        trace!("Removed job {:?}", n);
      }
    }
    true
  }


  #[inline]
  fn try_schedule_job(&mut self, node_index: NodeIndex, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    trace!("Try to schedule job {:?}", node_index);
    if let Some(JobStatus::Pending(_)) = self.job_graph.node_weight(node_index) {
      node_index_cache.clear();
      self.job_graph.neighbors_directed(node_index, Outgoing).collect_into(node_index_cache);
      if node_index_cache.iter().all(|n| self.job_graph[*n].is_completed()) {
        let mut dependency_outputs = DependencyOutputs::<D, O>::new();
        for dependency_node_index in node_index_cache.drain(..) {
          let dependency_output = self.job_graph[dependency_node_index].clone_completed();
          let dependency_edge_index = self.job_graph.find_edge(node_index, dependency_node_index).unwrap(); // Unwrap OK: edge exists.
          let dependency_edge = self.job_graph[dependency_edge_index];
          dependency_outputs.push((dependency_edge, dependency_output));
        }
        return self.schedule_job(node_index, dependency_outputs);
      }
    }
    true
  }

  #[inline]
  fn schedule_job(&mut self, node_index: NodeIndex, dependencies: DependencyOutputs<D, O>) -> bool {
    let job_status = &mut self.job_graph[node_index];
    if !job_status.is_pending() { return true; }
    trace!("Scheduling job {:?}", node_index);
    if let JobStatus::Pending(job_key) = std::mem::replace(job_status, JobStatus::Running) {
      self.pending_jobs -= 1;
      self.scheduled_jobs += 1;
      return self.to_worker.send((node_index, job_key, dependencies)).is_ok();
    }
    true
  }

  #[inline]
  fn complete_job(&mut self, node_index: NodeIndex, job_key: J, output: O) -> bool {
    trace!("Completing job {:?}", node_index);
    let wrapped = Arc::new(output);
    self.job_graph[node_index] = JobStatus::Completed(wrapped.clone());
    if self.to_queue.send(FromManagerMessage::JobCompleted(job_key, wrapped)).is_err() {
      return false;
    }
    self.scheduled_jobs -= 1;
    self.send_queue_empty_if_applicable()
  }

  #[inline]
  fn send_queue_empty_if_applicable(&mut self) -> bool {
    if self.pending_jobs == 0 && self.scheduled_jobs == 0 {
      return self.to_queue.send(FromManagerMessage::QueueEmpty).is_ok();
    }
    true
  }
}


// Job status

pub(super) enum JobStatus<J, O> {
  Pending(J),
  Running,
  Completed(Arc<O>),
}

impl<J, O> JobStatus<J, O> {
  #[inline]
  fn is_pending(&self) -> bool {
    match self {
      Self::Pending(_) => true,
      _ => false
    }
  }
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
