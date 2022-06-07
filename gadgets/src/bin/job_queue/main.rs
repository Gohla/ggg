use tracing::{info, trace};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::{Dependencies, DependencyOutputs, FromManagerMessage, JobQueue};

fn main() {
  profiling::register_thread!();
  dotenv::dotenv().ok();
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .init();

  let handler = move |key: i32, deps: DependencyOutputs<f32, f32>| {
    trace!("Executing job {} with deps {:?}", key, deps);
    let (dep, dep_output) = if deps.len() > 0 {
      let dep = &deps[0];
      (dep.0, *dep.1)
    } else {
      (1.0, 1.0)
    };
    key as f32 * (1.0 / dep_output) * dep
  };
  let job_queue = JobQueue::new(8, handler).unwrap();

  let mut prev = None;
  for key in 1i32..1024 {
    let dependencies = if let Some(prev) = prev {
      Dependencies::from_elem((prev as f32 * 2.0, prev), 1)
    } else {
      Dependencies::default()
    };
    trace!("Adding job with key {} and dependencies {:?}", key, dependencies);
    job_queue.add_job_with_dependencies(key, dependencies).unwrap();
    prev = Some(key);
  }
  job_queue.remove_job_and_dependencies(1023).unwrap();

  let receiver = job_queue.get_message_receiver();
  for message in receiver.iter() {
    match message {
      FromManagerMessage::JobCompleted(job_key, output) => info!("Completed job {}: {}", job_key, output),
      FromManagerMessage::QueueEmpty => {
        info!("Done!");
        break;
      }
    }
  }
  job_queue.stop_and_join().unwrap();
}
