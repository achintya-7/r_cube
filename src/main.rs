use std::error::Error;

use lib::worker::{
    types::{TaskServer, Worker},
    worker::run_tasks,
};
use std::sync::Arc;
use tokio::sync::Mutex;

mod lib {
    pub mod manager;
    pub mod tasks;
    pub mod worker;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let worker = Arc::new(Mutex::new(Worker::new("default_worker")));
    let worker_server = TaskServer::new(worker.clone(), "localhost", "8080");

    {
        let worker = worker.clone();
        tokio::spawn(async move { run_tasks(worker).await });
    }

    worker_server.start_server().await;

    Ok(())
}
