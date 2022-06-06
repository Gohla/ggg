use tracing::{info, trace};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::{Dependencies, DependencyOutputs, JobQueue};

fn main() {
  profiling::register_thread!();
  dotenv::dotenv().ok();
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .init();

  let handler = move |i: i32, deps: DependencyOutputs<(), i32>| {
    trace!("Executing job {} with deps {:?}", i, deps);
    let j = if deps.len() > 0 { *deps[0].1 } else { 1 };
    i.wrapping_mul(j)
  };
  let job_queue = JobQueue::new(8, handler).unwrap();
  
  let mut prev = None;
  for i in 1i32..1024 {
    let dependencies = if let Some(prev) = prev {
      Dependencies::from_elem(((), prev), 1)
    } else {
      Dependencies::default()
    };
    job_queue.add_job_with_dependencies(i, dependencies).unwrap();
    // trace!("Creating job for {}, with index {:?}", i, curr);
    prev = Some(i);
  }
  
  let receiver = job_queue.get_output_receiver();
  for (k, v) in receiver.iter() {
    info!("Completed job {}: {}", k, v);
    if receiver.is_empty() {
      info!("Done!");
      break;
    }
  }
  job_queue.stop_and_join().unwrap();
}
