use crossbeam_channel::{Receiver, select, Sender, unbounded};
use crossbeam_utils::thread;

fn main() {
  let (external_sender, external_receiver): (Sender<i32>, Receiver<i32>) = unbounded();

  thread::scope(|s| {
    let (manager_to_worker_sender, manager_to_worker_receiver) = unbounded();
    let (worker_to_manager_sender, worker_to_manager_receiver) = unbounded();
    s.spawn(move |_| {
      loop {
        select! {
          recv(external_receiver) -> task => {
            if let Ok(task) = task {
              println!("Submitting task {}", task);
              manager_to_worker_sender.send(task).unwrap();
            }
          },
          recv(worker_to_manager_receiver) -> value => {
            if let Ok(value) = value {
              println!("Completed task {}", value);
            }
          },
        }
      }
    });

    for _ in 0..8 {
      let manager_to_worker_receiver = manager_to_worker_receiver.clone();
      let worker_to_manager_sender = worker_to_manager_sender.clone();
      s.spawn(move |_| {
        loop {
          if let Ok(task) = manager_to_worker_receiver.recv() {
            let mut value = task;
            for i in 0..4096 {
              value = value.wrapping_add(value.wrapping_pow(i));
            }
            worker_to_manager_sender.send(value).unwrap();
          }
        }
      });
    }

    for i in 0..4096 {
      external_sender.send(i).unwrap();
    }
  }).unwrap();
}
