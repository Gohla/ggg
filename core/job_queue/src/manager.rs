use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, select, Sender};
use petgraph::prelude::*;
use rustc_hash::FxHashMap;
use tracing::trace;

use crate::{Dependencies, DependencyOutputs, DepKey, JobKey, Out};

// Message from queue

pub(crate) enum FromQueueMessage<J, D> {
  AddJob(J, Dependencies<J, D>),
  RemoveJob(J),
}

// Manager thread

pub(crate) type FromQueue<J, D> = FromQueueMessage<J, D>;
pub(crate) type FromWorker<J, O> = (NodeIndex, J, O);

pub(super) struct ManagerThread<J, D, O> {
  from_queue: Receiver<FromQueue<J, D>>,
  to_worker: Sender<crate::worker::FromManager<J, D, O>>,
  from_worker: Receiver<FromWorker<J, O>>,
  to_queue: Sender<super::FromManager<J, O>>,

  job_graph: StableDiGraph<JobStatus<J, O>, D>,
  job_key_to_node_index: FxHashMap<J, NodeIndex>,
}

impl<J: JobKey, D: DepKey, O: Out> ManagerThread<J, D, O> {
  #[inline]
  pub(super) fn new(
    from_queue: Receiver<FromQueue<J, D>>,
    to_worker: Sender<crate::worker::FromManager<J, D, O>>,
    from_worker: Receiver<FromWorker<J, O>>,
    to_queue: Sender<super::FromManager<J, O>>,
  ) -> Self {
    let job_graph = StableDiGraph::new();
    let job_key_to_node_index = FxHashMap::default();
    Self {
      from_queue,
      to_worker,
      from_worker,
      to_queue,
      job_graph,
      job_key_to_node_index,
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
          // trace!("Received job graph from the job queue");
          // 
          // self.job_graph = job_graph;
          // self.job_key_to_node_index = job_key_to_node_index;
          // 
          // // Schedule initial jobs.
          // self.job_graph.externals(Outgoing).collect_into(&mut node_index_cache_1);
          // for node_index in node_index_cache_1.drain(..) {
          //   if !self.schedule_job(node_index, Box::new([])) { 
          //     break; // All workers have disconnected; stop this thread.
          //   }
          // }
        },
        recv(self.from_worker) -> result => {
          let Ok((node_index, job_key, output)) = result else {
            break; // All workers have disconnected; stop this thread.
          };
          trace!("Received job {:?} output from worker", node_index);
          
          // TODO: handle the case where a running job was removed; not in the dependency graph any more!
          
          // Complete job
          if !self.complete_job(node_index, job_key, output) {
            // Job queue has disconnected; stop this thread.
          }
          
          // Try to schedule dependent jobs.
          self.job_graph.neighbors_directed(node_index, Incoming).collect_into(&mut node_index_cache_1);
          for dependent_node_index in node_index_cache_1.drain(..) {
            if !self.try_schedule(dependent_node_index, &mut node_index_cache_2) {
              break; // All workers have disconnected; stop this thread.
            }
          }
        },
      }
    }
    trace!("Stopped job queue manager thread");
  }

  fn handle_message(&mut self, message: FromQueueMessage<J, D>, node_index_cache: &mut Vec<NodeIndex>) -> bool {
    use FromQueueMessage::*;
    match message {
      AddJob(job_key, dependencies) => {
        if let Some(_node_index) = self.job_key_to_node_index.get(&job_key) {
          // Job already exists; do nothing.
          // TODO: update dependencies? when new ones are added, re-run the job?
        } else {
          let node_index = self.job_graph.add_node(JobStatus::Pending(job_key));
          for (dependency, dep_job_key) in dependencies.into_iter() {
            if let Some(dependency_node_index) = self.job_key_to_node_index.get(&dep_job_key) {
              self.job_graph.add_edge(node_index, *dependency_node_index, dependency);
            } else {
              panic!("Attempt to add dependency to job with key {:?} which has not been added", dep_job_key);
            }
          }
          self.job_key_to_node_index.insert(job_key, node_index);
          return self.try_schedule(node_index, node_index_cache);
        }
      }
      RemoveJob(job_key) => {
        if let Some(node_index) = self.job_key_to_node_index.remove(&job_key) {
          self.job_graph.remove_node(node_index);
          // TODO: also remove orphaned dependencies?
        }
      }
    }
    true
  }

  #[inline]
  fn try_schedule(&mut self, node_index: NodeIndex, node_index_cache: &mut Vec<NodeIndex>) -> bool {
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
      return self.to_worker.send((node_index, job_key, dependencies)).is_ok();
    }
    true
  }

  #[inline]
  fn complete_job(&mut self, node_index: NodeIndex, job_key: J, output: O) -> bool {
    trace!("Completing job {:?}", node_index);
    let wrapped = Arc::new(output);
    self.job_graph[node_index] = JobStatus::Completed(wrapped.clone());
    self.to_queue.send((job_key, wrapped)).is_ok()
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
