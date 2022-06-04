use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, select, Sender, unbounded};

pub struct JobQueue<K, D, F, V> {
  manager_thread: JoinHandle<()>,
  worker_threads: Vec<JoinHandle<()>>,
  external_to_manager_sender: Sender<(K, D, F)>,
  manager_to_external_receiver: Receiver<(K, V)>,
}

impl<K: Copy + Send + 'static, D: Send + 'static, F: FnOnce(K, D) -> V + Send + 'static, V: Send + 'static> JobQueue<K, D, F, V> {
  pub fn new(worker_thread_count: usize) -> Self {
    let (external_to_manager_sender, external_to_manager_receiver): (Sender<(K, D, F)>, Receiver<(K, D, F)>) = unbounded();
    let (manager_to_worker_sender, manager_to_worker_receiver): (Sender<(K, D, F)>, Receiver<(K, D, F)>) = unbounded();
    let (worker_to_manager_sender, worker_to_manager_receiver) = unbounded();
    let (manager_to_external_sender, manager_to_external_receiver) = unbounded();

    let manager_thread = {
      thread::Builder::new()
        .name("Job Queue Manager".into())
        .spawn(move || {
          loop {
            select! {
              recv(external_to_manager_receiver) -> job => {
                let job = job.unwrap(); // Unwrap OK: panics iff external->manager sender is dropped.
                manager_to_worker_sender.send(job).unwrap(); // Unwrap OK: panics iff all manager->worker receivers are dropped.
              },
              recv(worker_to_manager_receiver) -> result => {
                let result = result.unwrap(); // Unwrap OK: panics iff all worker->manager senders are dropped.
                manager_to_external_sender.send(result).unwrap(); // Unwrap OK: panics iff all manager->external receivers are dropped.
              },
            }
          }
        }).unwrap() // Unwrap OK?: panic iff creating thread fails.
    };

    let mut worker_threads = Vec::new();
    for i in 0..worker_thread_count {
      let manager_to_worker_receiver = manager_to_worker_receiver.clone();
      let worker_to_manager_sender = worker_to_manager_sender.clone();
      let worker_thread = thread::Builder::new()
        .name(format!("Job Queue Worker {}", i))
        .spawn(move || {
          loop {
            let (key, data, function) = manager_to_worker_receiver.recv().unwrap(); // Unwrap OK: panics iff manager->worker sender is dropped.
            let value = function(key, data);
            worker_to_manager_sender.send((key, value)).unwrap(); // Unwrap OK: panics iff worker->manager receiver is dropped.
          }
        }).unwrap(); // Unwrap OK?: panic iff creating thread fails.
      worker_threads.push(worker_thread);
    }

    Self {
      manager_thread,
      worker_threads,
      external_to_manager_sender,
      manager_to_external_receiver,
    }
  }

  pub fn add(&self, key: K, data: D, function: F) {
    self.external_to_manager_sender.send((key, data, function)).unwrap(); // Unwrap OK: panics iff external->manager receiver is dropped.
  }

  pub fn join(self) -> thread::Result<()> {
    self.manager_thread.join()?;
    for worker_thread in self.worker_threads {
      worker_thread.join()?;
    }
    Ok(())
  }

  pub fn get_value_receiver(&self) -> &Receiver<(K, V)> { &self.manager_to_external_receiver }
}
