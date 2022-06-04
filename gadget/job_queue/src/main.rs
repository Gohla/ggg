use petgraph::prelude::*;

use job_queue::{Job, JobQueue};

fn main() {
  let job_queue = JobQueue::new(8).unwrap();
  let mut job_graph = Graph::new();
  let mut prev = NodeIndex::default();
  for i in (0..4096).rev() {
    let job = Job::new(move || i * i);
    let curr = job_graph.add_node(Some(job));
    job_graph.add_edge(prev, curr, ());
    prev = curr;
  }
  job_queue.set_job_graph(job_graph);
  for v in job_queue.get_value_receiver().iter() {
    println!("Completed job {}", v);
  }
  job_queue.join().unwrap();
}
