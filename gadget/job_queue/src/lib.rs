#![feature(iter_collect_into)]

use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, select, Sender, unbounded};
use petgraph::prelude::*;

pub struct JobQueue<F: FnMut() -> O, O> {
  manager_thread_handle: JoinHandle<()>,
  worker_thread_handles: Vec<JoinHandle<()>>,
  external_to_manager_sender: Sender<Graph<Option<Job<F, O>>, ()>>,
  manager_to_external_receiver: Receiver<O>,
}

impl<F: FnMut() -> O + Send + 'static, O: Send + 'static> JobQueue<F, O> {
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
      let manager_to_worker_receiver = manager_to_worker_receiver.clone();
      let worker_to_manager_sender = worker_to_manager_sender.clone();
      let worker_thread = WorkerThread {
        from_manager: manager_to_worker_receiver,
        to_manager: worker_to_manager_sender,
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

  pub fn set_job_graph(&self, job_graph: Graph<Option<Job<F, O>>, ()>) {
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

pub struct Job<F: FnMut() -> O, O> {
  function: F,
}

impl<F: FnMut() -> O, O> Job<F, O> {
  #[inline]
  pub fn new(function: F) -> Self { Self { function } }
  #[inline]
  fn run(mut self) -> O { (self.function)() }
}

// Manager thread

struct ManagerThread<F: FnMut() -> O, O> {
  from_external: Receiver<Graph<Option<Job<F, O>>, ()>>,
  to_worker: Sender<(Job<F, O>, NodeIndex)>,
  from_worker: Receiver<(O, NodeIndex)>,
  to_external: Sender<O>,
  job_graph: Graph<Option<Job<F, O>>, ()>,
  nodes_to_schedule_cache: Vec<NodeIndex>,
}

impl<F: FnMut() -> O + Send + 'static, O: Send + 'static> ManagerThread<F, O> {
  fn new(
    from_external: Receiver<Graph<Option<Job<F, O>>, ()>>,
    to_worker: Sender<(Job<F, O>, NodeIndex)>,
    from_worker: Receiver<(O, NodeIndex)>,
    to_external: Sender<O>,
  ) -> Self {
    let job_graph = Graph::new();
    let nodes_to_schedule_cache = Vec::new();
    Self {
      from_external,
      to_worker,
      from_worker,
      to_external,
      job_graph,
      nodes_to_schedule_cache,
    }
  }

  fn create_thread_and_run(self) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name("Job Queue Manager".into())
      .spawn(|| { self.run() })
  }

  fn run(mut self) {
    loop {
      select! {
        recv(self.from_external) -> job_graph => {
          let job_graph = job_graph.unwrap(); // Unwrap OK: panics iff external->manager sender is dropped.
          self.job_graph = job_graph;
          self.schedule_jobs();
        },
        recv(self.from_worker) -> result => {
          let (output, node_index) = result.unwrap(); // Unwrap OK: panics iff all worker->manager senders are dropped.
          self.to_external.send(output).unwrap(); // Unwrap OK: panics iff all manager->external receivers are dropped.
          self.job_graph.remove_node(node_index);
          self.schedule_jobs();
        },
      }
    }
  }

  fn schedule_jobs(&mut self) {
    self.nodes_to_schedule_cache.clear();
    self.job_graph.externals(Outgoing).collect_into(&mut self.nodes_to_schedule_cache);
    for node_index in &self.nodes_to_schedule_cache {
      if let Some(job) = self.job_graph.node_weight_mut(*node_index).unwrap().take() { // Unwrap OK: node must exist.
        self.to_worker.send((job, *node_index)).unwrap(); // Unwrap OK: panics iff all manager->worker receivers are dropped.
      }
    }
  }
}

// Worker thread

struct WorkerThread<F: FnMut() -> O, O> {
  from_manager: Receiver<(Job<F, O>, NodeIndex)>,
  to_manager: Sender<(O, NodeIndex)>,
}

impl<F: FnMut() -> O + Send + 'static, O: Send + 'static> WorkerThread<F, O> {
  fn create_thread_and_run(self, thread_index: usize) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
      .name(format!("Job Queue Worker {}", thread_index))
      .spawn(|| { self.run() })
  }

  fn run(self) {
    loop {
      let (job, index) = self.from_manager.recv().unwrap(); // Unwrap OK: panics iff manager->worker sender is dropped.
      let output = job.run();
      self.to_manager.send((output, index)).unwrap(); // Unwrap OK: panics iff worker->manager receiver is dropped.
    }
  }
}
