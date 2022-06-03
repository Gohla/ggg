use std::iter;
use std::time::Duration;

use crossbeam_deque::{Injector, Stealer, Worker};
use crossbeam_utils::{Backoff, thread};

fn main() {
  let global = Injector::new();

  let mut workers = Vec::new();
  let mut stealers = Vec::new();
  for _ in 0..8 {
    let worker = Worker::new_fifo();
    let stealer = worker.stealer();
    workers.push(worker);
    stealers.push(stealer);
  }

  thread::scope(|s| {
    let mut threads = Vec::new();
    for (i, local) in workers.into_iter().enumerate() {
      let stealers: Vec<_> = stealers.iter().cloned().enumerate().filter_map(|(idx, s)| (i != idx).then_some(s)).collect();
      threads.push(s.spawn(|_| {
        let local = local;
        let stealers = stealers;
        let backoff = Backoff::new();
        loop {
          if let Some(task) = find_task(&local, &global, &stealers) {
            println!("{}", task);
            backoff.reset();
          } else if backoff.is_completed() {
            std::thread::park()
          } else {
            backoff.snooze();
          }
        }
      }));
    }

    for i in 0..4096 {
      global.push(i);
    }
    for thread in &threads {
      thread.thread().unpark();
    }

    std::thread::sleep(Duration::from_secs(10));
    
    for i in 4096..8192 {
      global.push(i);
    }
    for thread in &threads {
      thread.thread().unpark();
    }
  }).unwrap();
}

#[inline]
fn find_task<T>(
  local: &Worker<T>,
  global: &Injector<T>,
  stealers: &[Stealer<T>],
) -> Option<T> {
  local.pop().or_else(|| {
    iter::repeat_with(|| {
      global.steal_batch_and_pop(local)
        .or_else(|| stealers.iter().map(|s| s.steal()).collect())
    })
      .find(|s| !s.is_retry())
      .and_then(|s| s.success())
  })
}
