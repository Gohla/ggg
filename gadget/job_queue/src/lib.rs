#![feature(iter_collect_into)]
#![feature(let_else)]

use std::marker::PhantomData;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, select, Sender, unbounded};
use petgraph::prelude::*;
use tracing::trace;

pub struct JobQueue<F, O, E = ()> {
  manager_thread_handle: JoinHandle<()>,
  worker_thread_handles: Vec<JoinHandle<()>>,
  external_to_manager_sender: Sender<StableGraph<JobStatus<F, O, E>, E>>,
  manager_to_external_receiver: Receiver<O>,
}

impl<F: FnMut(Box<[(Arc<O>, E)]>) -> O + Send + 'static, O: Send + Sync + 'static, E: Copy + Send + 'static> JobQueue<F, O, E> {
  pub fn new(worker_thread_count: usize) -> std::io::Result<Self> {
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
      let worker_thread = WorkerThread {
        from_manager,
        to_manager,
      };
      let worker_thread_handle = worker_thread.create_thread_and_run(i)?;
      worker_thread_handles.push(worker_thread_handle);
    }

    Ok(Self {
      manager_thread_handle,
      worker_thread_handles,
      external_to_manager_sender,
      manager_to_external_receiver,
    })
  }

  pub fn set_job_graph(&self, job_graph: StableGraph<JobStatus<F, O, E>, E>) {
    self.external_to_manager_sender.send(job_graph).unwrap(); // Unwrap OK: panics iff external->manager receiver is dropped.
  }

  pub fn join(self) -> thread::Result<()> {
    self.manager_thread_handle.join()?;
    for worker_thread in self.worker_thread_handles {
      worker_thread.join()?;
    }
    Ok(())
  }

  pub fn get_value_receiver(&self) -> &Receiver<O> { &self.manager_to_external_receiver }
}

// Job

#[repr(transparent)]
pub struct Job<F, O, E = ()> {
  function: F,
  _output_phantom: PhantomData<O>,
  _edge_phantom: PhantomData<E>,
}

impl<F: FnMut(Box<[(Arc<O>, E)]>) -> O, O, E> Job<F, O, E> {
  #[inline]
  pub fn new(function: F) -> Self {
    Self {
      function,
      _output_phantom: Default::default(),
      _edge_phantom: Default::default(),
    }
  }

  #[inline]
  fn run(mut self, dependencies: Box<[(Arc<O>, E)]>) -> O {
    (self.function)(dependencies)
  }
}

pub enum JobStatus<F, O, E = ()> {
  Pending(Job<F, O, E>),
  Running,
  Wrapped(Arc<O>),
}

impl<F, O, E> JobStatus<F, O, E> {
  #[inline]
  fn is_pending(&self) -> bool {
    match self {
      Self::Pending(_) => true,
      _ => false
    }
  }

  #[inline]
  fn is_wrapped(&self) -> bool {
    match self {
      Self::Wrapped(_) => true,
      _ => false
    }
  }

  #[inline]
  fn clone_wrapped(&self) -> Arc<O> {
    match self {
      Self::Wrapped(arc) => arc.clone(),
      _ => panic!("Attempt to clone_wrapped on non-Wrapped job status")
    }
  }

  #[inline]
  fn unwrap(self) -> O {
    match self {
      Self::Wrapped(arc) => Arc::try_unwrap(arc).unwrap_or_else(|_| panic!("Attempt to unwrap shared Arc")),
      _ => panic!("Attempt to call unwrap on non-Wrapped job status")
    }
  }
}

// Manager thread

struct ManagerThread<F, O, E = ()> {
  from_external: Receiver<StableGraph<JobStatus<F, O, E>, E>>,
  to_worker: Sender<(Job<F, O, E>, Box<[(Arc<O>, E)]>, NodeIndex)>,
  from_worker: Receiver<(O, NodeIndex)>,
  to_external: Sender<O>,
  job_graph: StableGraph<JobStatus<F, O, E>, E>,
}

impl<F: FnMut(Box<[(Arc<O>, E)]>) -> O + Send + 'static, O: Send + Sync + 'static, E: Copy + Send + 'static> ManagerThread<F, O, E> {
  fn new(
    from_external: Receiver<StableGraph<JobStatus<F, O, E>, E>>,
    to_worker: Sender<(Job<F, O, E>, Box<[(Arc<O>, E)]>, NodeIndex)>,
    from_worker: Receiver<(O, NodeIndex)>,
    to_external: Sender<O>,
  ) -> Self {
    let job_graph = StableGraph::new();
    Self {
      from_external,
      to_worker,
      from_worker,
      to_external,
      job_graph,
    }
  }

  fn create_thread_and_run(self) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name("Job Queue Manager".into())
      .spawn(|| { self.run() })
  }

  fn run(mut self) {
    trace!("Started job queue manager thread");
    let mut node_index_cache_1 = Vec::new();
    let mut node_index_cache_2 = Vec::new();
    loop {
      select! {
        recv(self.from_external) -> job_graph => {
          let Ok(job_graph) = job_graph else {
             break; // Job queue was dropped; stop this thread.
          };
          trace!("Received job graph from the job queue");
          
          self.job_graph = job_graph;
          
          // Schedule initial jobs.
          node_index_cache_1.clear();
          self.job_graph.externals(Outgoing).collect_into(&mut node_index_cache_1);
          for node_index in node_index_cache_1.drain(..) {
            if !self.schedule_job(node_index, Box::new([])) { 
              break; // All workers have disconnected; stop this thread.
            }
          }
        },
        recv(self.from_worker) -> result => {
          let Ok((output, node_index)) = result else {
            break; // All workers have disconnected; stop this thread.
          };
          trace!("Received job {:?} output from worker", node_index);
          
          // Update node weight to wrapped output.
          self.job_graph[node_index] = JobStatus::Wrapped(Arc::new(output));
          
          // Try to schedule dependent jobs.
          let mut can_complete_this_job = true;
          node_index_cache_1.clear();
          self.job_graph.neighbors_directed(node_index, Incoming).collect_into(&mut node_index_cache_1);
          for dependent_node_index in node_index_cache_1.drain(..) {
            trace!("Try to schedule dependent job {:?}", dependent_node_index);
            if let Some(JobStatus::Pending(_)) = self.job_graph.node_weight(dependent_node_index) {
              can_complete_this_job = false;
              node_index_cache_2.clear();
              self.job_graph.neighbors_directed(dependent_node_index, Outgoing).collect_into(&mut node_index_cache_2);
              if node_index_cache_2.iter().all(|n|self.job_graph.node_weight(*n).unwrap().is_wrapped()) { // Unwrap OK: node exists.
                let mut dependency_outputs = Vec::new(); // OPTO: smallvec?
                for dependency_node_index in node_index_cache_2.drain(..) {
                  let dependency_output = self.job_graph.node_weight(dependency_node_index).unwrap().clone_wrapped(); // Unwrap OK: node exists.
                  let dependency_edge_index = self.job_graph.find_edge(dependent_node_index, dependency_node_index).unwrap();
                  let dependency_edge = *self.job_graph.edge_weight(dependency_edge_index).unwrap(); // Unwrap OK: edge exists.
                  dependency_outputs.push((dependency_output, dependency_edge));
                }
                if !self.schedule_job(dependent_node_index, dependency_outputs.into_boxed_slice()) {
                  break; // All workers have disconnected; stop this thread.
                }
              }
            }
          }
          
          // Cache dependencies before trying to complete this job, as completing it removes it from the graph.
          node_index_cache_1.clear();
          self.job_graph.neighbors_directed(node_index, Outgoing).collect_into(&mut node_index_cache_1);
          
          // Try to complete this job.
          if can_complete_this_job { // OPTO: we wrap above and may immediately unwrap an Arc here; prevent that.
            if !self.complete_job(node_index) {
              break; // All workers have disconnected; stop this thread.
            }
          }
          
          // Try to complete dependency jobs.
          for dependency_node_index in node_index_cache_1.drain(..) {
            trace!("Try to complete dependency job {:?}", dependency_node_index);
            let is_wrapped = self.job_graph[dependency_node_index].is_wrapped();
            let all_dependents_wrapped = self.job_graph.neighbors_directed(dependency_node_index, Incoming).all(|n|self.job_graph[n].is_wrapped());
            if is_wrapped && all_dependents_wrapped {
              if !self.complete_job(dependency_node_index) {
                break; // All workers have disconnected; stop this thread.
              }
            }
          }
        },
      }
    }
    trace!("Stopped job queue manager thread");
  }

  #[inline]
  fn schedule_job(&mut self, node_index: NodeIndex, dependencies: Box<[(Arc<O>, E)]>) -> bool {
    let job_status = &mut self.job_graph[node_index];
    if !job_status.is_pending() { return true; }
    if let JobStatus::Pending(job) = std::mem::replace(job_status, JobStatus::Running) {
      trace!("Scheduling job {:?}", node_index);
      if self.to_worker.send((job, dependencies, node_index)).is_err() {
        return false; // All workers have disconnected; return false indicating that the manager should stop.
      }
    }
    true
  }

  #[inline]
  fn complete_job(&mut self, node_index: NodeIndex) -> bool {
    let job_status_wrapped = self.job_graph.remove_node(node_index).unwrap(); // Unwrap OK: node exists.
    // Unwrap OK: it is wrapped and the only owner of the Arc.
    let output = job_status_wrapped.unwrap();
    trace!("Completing job {:?}", node_index);
    self.to_external.send(output).is_ok()
  }
}

// Worker thread

struct WorkerThread<F, O, E = ()> {
  from_manager: Receiver<(Job<F, O, E>, Box<[(Arc<O>, E)]>, NodeIndex)>,
  to_manager: Sender<(O, NodeIndex)>,
}

impl<F: FnMut(Box<[(Arc<O>, E)]>) -> O + Send + 'static, O: Send + Sync + 'static, E: Send + 'static> WorkerThread<F, O, E> {
  fn create_thread_and_run(self, thread_index: usize) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name(format!("Job Queue Worker {}", thread_index))
      .spawn(move || { self.run(thread_index) })
  }

  fn run(self, thread_index: usize) {
    trace!("Started job queue worker thread {}", thread_index);
    loop {
      if let Ok((job, dependencies, node_index)) = self.from_manager.recv() {
        trace!("Running job {:?}", node_index);
        let output = job.run(dependencies);
        if self.to_manager.send((output, node_index)).is_err() {
          break; // Manager has disconnected; stop this thread.
        }
      } else {
        break; // Manager has disconnected; stop this thread.
      }
    }
    trace!("Stopped job queue worker thread {}", thread_index);
  }
}
