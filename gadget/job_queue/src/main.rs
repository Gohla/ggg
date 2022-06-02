use std::iter;

use crossbeam_deque::{Injector, Stealer, Worker};
use crossbeam_utils::thread;

fn main() {
  let injector = Injector::new();

  let mut workers = Vec::new();
  let mut stealers = Vec::new();
  for _ in 0..8 {
    let worker = Worker::new_fifo();
    let stealer = worker.stealer();
    workers.push(worker);
    stealers.push(stealer);
  }

  thread::scope(|s| {
    for (i, worker) in workers.into_iter().enumerate() {
      let stealers: Vec<_> = stealers.iter().cloned().enumerate().filter_map(|(idx, s)| (i != idx).then_some(s)).collect();
      s.spawn(|_| {
        let worker = worker;
        let stealers = stealers;
        loop {
          if let Some(task) = find_task(&worker, &injector, &stealers) {
            println!("{}", task);
          }
        }
      });
    }

    for i in 0..4096 {
      injector.push(i);
    }
  }).unwrap();
}

fn find_task<T>(
  local: &Worker<T>,
  global: &Injector<T>,
  stealers: &[Stealer<T>],
) -> Option<T> {
  // Pop a task from the local queue, if not empty.
  local.pop().or_else(|| {
    // Otherwise, we need to look for a task elsewhere.
    iter::repeat_with(|| {
      // Try stealing a batch of tasks from the global queue.
      global.steal_batch_and_pop(local)
        // Or try stealing a task from one of the other threads.
        .or_else(|| stealers.iter().map(|s| s.steal()).collect())
    })
      // Loop while no task was stolen and any steal operation needs to be retried.
      .find(|s| !s.is_retry())
      // Extract the stolen task, if there is one.
      .and_then(|s| s.success())
  })
}
