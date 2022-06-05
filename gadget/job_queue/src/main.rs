use petgraph::prelude::*;
use tracing::info;
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
  for i in (0..4096).rev() {
    let job = Job::<_, _, ()>::new(move |_| (i, i * i));
    let curr = job_graph.add_node(JobStatus::Pending(job));
    if let Some(prev) = prev {
      job_graph.add_edge(prev, curr, ());
    }
    prev = Some(curr);
  }
  job_queue.set_job_graph(job_graph);
  for (k, v) in job_queue.get_value_receiver().iter() {
    info!("Completed job {}: {}", k, v);
  }
  job_queue.join().unwrap();
}
