use std::{error::Error, fmt::format, vec};

use lib::worker::{
    types::{TaskServer, Worker},
    worker::{collect_stats, run_tasks},
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::lib::{
    manager::types::Manager,
    tasks::types::{State, Task, TaskEvent},
};

mod lib {
    pub mod manager;
    pub mod tasks;
    pub mod worker;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let worker = Arc::new(Mutex::new(Worker::new("default_worker")));
    let worker_server = TaskServer::new(worker.clone(), "localhost", "8080");
    let workers = vec![format!("{}:{}", worker_server.address, worker_server.port)];
    let manager = Manager::new(workers);

    {
        let worker = worker.clone();
        let sysinfo_worker = worker.clone();
        tokio::spawn(async move {
            let stats_task = collect_stats(worker);
            let tasks_task = run_tasks(sysinfo_worker);
            tokio::join!(stats_task, tasks_task);
        });
    }

    // Anonymous async block to wait 2 seconds before adding tasks
    tokio::spawn({
        let mut manager = manager.clone();
        async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            for i in 0..3 {
                let mut task = Task::default();
                task.id = format!("task_{}", i);
                task.state = State::Scheduled;
                task.image = "hello-world:latest".to_string();
                task.name = format!("Test Container {}", i);

                let mut task_event = TaskEvent::default();

                task_event.task_id = uuid::Uuid::new_v4().to_string();
                task_event.event_type = "running".to_string();
                task_event.timestamp = Some(std::time::SystemTime::now());
                task_event.task = task.clone();

                manager.add_task(task.clone());
                manager.send_work().await;
            }
        }
    });

    worker_server.start_server().await;

    Ok(())
}
