use tracing::{info, trace};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::{Dependencies, DependencyOutputs, JobQueue};

fn main() {
  profiling::register_thread!();
  dotenv::dotenv().ok();
  let filter_layer = EnvFilter::from_env("MAIN_LOG");
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .init();

  let handler = move |key: i32, deps: DependencyOutputs<f32, f32, 1>, input: f32| {
    trace!("Executing job {} with deps {:?}", key, deps);
    let (dep, dep_output) = if deps.len() > 0 {
      let dep = &deps[0];
      (dep.0, *dep.1)
    } else {
      (1.0, 1.0)
    };
    key as f32 * input * (1.0 / dep_output) * dep
  };
  let job_queue = JobQueue::new(8, handler).unwrap();

  let mut prev = None;
  for key in 1i32..1024 {
    let dependencies = if let Some(prev) = prev {
      Dependencies::from_elem((prev as f32 / 2.0, prev), 1)
    } else {
      Dependencies::default()
    };
    trace!("Adding job with key {} and dependencies {:?}", key, dependencies);
    job_queue.try_add_job_with_dependencies(key, dependencies, key as f32 * 2.0).unwrap();
    prev = Some(key);
  }
  job_queue.try_remove_job_and_orphaned_dependencies(1023).unwrap();

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
