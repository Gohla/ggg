use std::sync::Arc;

use petgraph::prelude::*;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::{Job, JobQueue, JobStatus};

fn main() {
  profiling::register_thread!();

  dotenv::dotenv().ok();

  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .init();

  let job_queue = JobQueue::new(8).unwrap();
  let mut job_graph = StableGraph::new();
  let mut prev = None;
  for i in 0i32..1024 {
    let job = Job::new(move |deps: Box<[(Arc<(i32, i32)>, i32)]>| {
      info!("Executing job {} with deps {:?}", i, deps);
      let j = if deps.len() > 0 { deps[0].1 } else { 1 };
      (i, i.wrapping_mul(j))
    });
    let curr = job_graph.add_node(JobStatus::Pending(job));
    debug!("Creating job for {}, with index {:?}", i, curr);
    if let Some(prev) = prev {
      debug!("Adding dependency edge from {:?} to {:?} with value {}", prev, curr, i);
      job_graph.add_edge(prev, curr, i);
    }
    prev = Some(curr);
  }
  job_queue.set_job_graph(job_graph);
  for (k, v) in job_queue.get_value_receiver().iter() {
    info!("Completed job {}: {}", k, v);
  }
  job_queue.join().unwrap();
}
