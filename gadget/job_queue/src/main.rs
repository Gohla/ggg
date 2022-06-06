use std::sync::Arc;

use petgraph::prelude::*;
use tracing::{trace, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use job_queue::{JobQueue, JobStatus};

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
    let job = move |deps: Box<[(Arc<(i32, i32)>, i32)]>| {
      trace!("Executing job {} with deps {:?}", i, deps);
      let j = if deps.len() > 0 { deps[0].1 } else { 1 };
      (i, i.wrapping_mul(j))
    };
    let curr = job_graph.add_node(JobStatus::Pending(job));
    trace!("Creating job for {}, with index {:?}", i, curr);
    if let Some(prev) = prev {
      trace!("Adding dependency edge from {:?} to {:?} with value {}", prev, curr, i);
      job_graph.add_edge(prev, curr, i);
    }
    prev = Some(curr);
  }
  job_queue.set_job_graph(job_graph).unwrap();
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
