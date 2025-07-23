use sysinfo::System;
use tokio::sync::Mutex;

use super::types::Worker;
use crate::lib::{
    tasks::{
        state::valid_state_transition,
        types::{DockerClient, DockerResult, State, Task, new_config},
    },
    worker::types::{SystemStats, get_stats},
};
use std::{
    io::{Error, ErrorKind::Other},
    sync::Arc,
    time::SystemTime,
};

impl Worker {
    pub fn new(name: &str) -> Self {
        let sys = System::new_all();
        Worker {
            name: name.to_string(),
            queue: std::collections::VecDeque::new(),
            db: std::collections::HashMap::new(),
            task_count: 0,
            sysinfo: sys,
        }
    }

    async fn run_task(&mut self) -> DockerResult {
        let task = match self.queue.pop_front() {
            Some(task) => task,
            None => {
                println!("No tasks in queue");
                return DockerResult::with_error(Box::new(Error::new(Other, "No tasks in queue")));
            }
        };

        let persisted = self
            .db
            .entry(task.id.clone())
            .or_insert_with(|| Box::new(task.clone()));

        if !valid_state_transition(&persisted.state, &task.state) {
            println!(
                "Invalid state transition from {:?} to {:?}",
                persisted.state, task.state
            );
            return DockerResult::with_error(Box::new(Error::new(
                Other,
                "Invalid state transition",
            )));
        }

        match task.state {
            State::Scheduled => {
                println!("Task is scheduled, starting it now");
                self.start_task(task).await
            }
            State::Completed => {
                println!("Task is completed, stopping it now");
                self.stop_task(task).await
            }
            _ => {
                println!(
                    "Invalid state for task: {:?} with id: {:?}",
                    task.state, task.id
                );
                DockerResult::with_error(Box::new(Error::new(Other, "Invalid state for task")))
            }
        }
    }

    async fn start_task(&mut self, mut task: Task) -> DockerResult {
        task.start_time = Some(SystemTime::now());
        let config = new_config(task.clone());

        let docker_client = match DockerClient::new(config) {
            Some(client) => client,
            None => {
                println!("Failed to create Docker client");
                return DockerResult::with_error(Box::new(Error::new(
                    Other,
                    "Failed to create Docker client",
                )));
            }
        };

        let result = docker_client.run().await;
        if result.error.is_some() {
            println!("Error running task: {:?}", result.error);
            task.state = State::Failed;
            return result;
        }
        println!(
            "Task started successfully with container ID: {:?}",
            result.container_id
        );

        if let Some(container_id) = result.container_id.clone() {
            task.finish_time = Some(SystemTime::now());
            task.state = State::Running;
            task.container_id = Some(container_id);
            self.db.insert(task.id.clone(), Box::new(task.clone()));
        }
        result
    }

    pub fn add_task(&mut self, task: Task) {
        self.queue.push_back(task);
    }

    async fn stop_task(&mut self, mut task: Task) -> DockerResult {
        let config = new_config(task.clone());
        let docker_client = match DockerClient::new(config) {
            Some(client) => client,
            None => {
                println!("Failed to create Docker client");
                return DockerResult::with_error(Box::new(Error::new(
                    Other,
                    "Failed to create Docker client",
                )));
            }
        };

        let container_id = match task.container_id.clone() {
            Some(id) => id,
            None => {
                println!("No container_id for task");
                return DockerResult::with_error(Box::new(Error::new(
                    Other,
                    "No container_id for task",
                )));
            }
        };

        let result = docker_client.stop(&container_id).await;
        if result.error.is_some() {
            println!("Error stopping task: {:?}", result.error);
            return result;
        }

        task.state = State::Completed;
        task.finish_time = Some(SystemTime::now());

        self.db.insert(task.id.clone(), Box::new(task.clone()));
        println!(
            "Stopped and removed task with container ID: {:?}",
            result.container_id
        );
       
        result
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.db.values().map(|task| task.as_ref().clone()).collect()
    }
}

pub async fn run_tasks(worker: Arc<Mutex<Worker>>) {
    loop {
        if !worker.lock().await.queue.is_empty() {
            match worker.lock().await.run_task().await {
                DockerResult {
                    error: Some(err), ..
                } => {
                    println!("Error running task: {:?}", err);
                }
                DockerResult { .. } => {
                    println!("Task completed successfully");
                }
            }
        } else {
            println!("No tasks in queue, waiting...");
        }

        // Sleep for a while before checking the queue again
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub async fn collect_stats(worker: Arc<Mutex<Worker>>) {
    loop {
        println!("Collecting system stats... ");
        let mut worker_guard = worker.lock().await;
        worker_guard.sysinfo.refresh_all();
        let stats = get_stats(&worker_guard.sysinfo, worker_guard.task_count);
        println!("System Stats: {}", serde_json::to_string(&stats).unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub async fn get_system_stats(worker: Arc<Mutex<Worker>>) -> SystemStats {
    let mut worker_guard = worker.lock().await;
    worker_guard.sysinfo.refresh_all();
    get_stats(&worker_guard.sysinfo, worker_guard.task_count)
}
