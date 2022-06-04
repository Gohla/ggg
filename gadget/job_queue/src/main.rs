use job_queue::JobQueue;

fn main() {
  let job_queue = JobQueue::new(8);
  for i in 0..4096 {
    job_queue.add(i, i as f32, |_k, d| {
      d * d
    });
  }
  for (k, v) in job_queue.get_value_receiver().iter() {
    println!("Completed job {} => {}", k, v);
  }
  job_queue.join().unwrap();
}
