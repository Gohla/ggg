use std::iter::FusedIterator;

use tracing::{info, trace};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::JobQueue;

fn main() {
  profiling::register_thread!();
  dotenv::dotenv().ok();
  let filter_layer = EnvFilter::from_env("MAIN_LOG");
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .init();

  let job_queue = JobQueue::new(
    8,
    1024,
    1024,
    move |key: i32, input: f32, deps: &[(f32, f32)]| {
      trace!("Executing job {} with deps {:?}", key, deps);
      let (dependency_key, dependency_output) = if deps.len() > 0 {
        let dep = &deps[0];
        (dep.0, dep.1)
      } else {
        (1.0, 1.0)
      };
      key as f32 * input * (1.0 / dependency_output) * dependency_key
    }).unwrap();

  job_queue.try_add_job(Job(1024)).unwrap();

  let receiver = job_queue.get_message_receiver();
  let mut done = false;
  for message in receiver.iter() {
    use job_queue::JobQueueMessage::*;
    match message {
      JobCompleted(job_key, output) => info!("Completed job {}: {}", job_key, output),
      PendingJobRemoved(job_key, input) => info!("Pending job {} with input {} removed", job_key, input),
      RunningJobRemoved(job_key) => info!("Running job {} removed", job_key),
      CompletedJobRemoved(job_key, output) => info!("Completed job {} with output {:?} removed", job_key, output),
      QueueEmpty => {
        done = true;
        info!("Done!");
      }
    }
    if done && receiver.is_empty() { break; }
  }
  job_queue.stop_and_join().unwrap();
}


// Job implementation

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct Job(i32);

impl job_queue::Job<i32, f32, f32> for Job {
  fn key(&self) -> &i32 { &self.0 }

  type DependencyIterator = JobDependencyIterator;

  fn into(self) -> (f32, Self::DependencyIterator) {
    let input = self.0 as f32 * 2.0;
    let dependencies = JobDependencyIterator(Some(self.0));
    (input, dependencies)
  }
}

#[repr(transparent)]
struct JobDependencyIterator(Option<i32>);

impl Iterator for JobDependencyIterator {
  type Item = (f32, Job);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match self.0 {
      Some(job_key) if job_key > 1 => {
        let previous_job_key = job_key - 1;
        let dependency_key = previous_job_key as f32 / 2.0;
        self.0 = None;
        Some((dependency_key, Job(previous_job_key)))
      },
      _ => None,
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let count = self.len();
    (count, Some(count))
  }

  #[inline]
  fn count(self) -> usize where Self: Sized {
    self.len()
  }
}

impl FusedIterator for JobDependencyIterator {}

impl ExactSizeIterator for JobDependencyIterator {
  #[inline]
  fn len(&self) -> usize {
    match self.0 {
      Some(job_key) if job_key > 1 => 1,
      _ => 0,
    }
  }
}
