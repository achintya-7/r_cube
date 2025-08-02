use error_stack::Report;
use sysinfo::System;
use tokio::sync::Mutex;

use super::types::Worker;
use crate::lib::{
    tasks::{
        state::valid_state_transition,
        types::{new_config, DockerClient, DockerError, DockerResult, State, Task},
    },
    worker::{stats::get_stats, types::{SystemStats, WorkerError}},
};
use std::{error::Error, fmt, sync::Arc, time::SystemTime};

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
                let error_report = Report::new(WorkerError::NoTasksInQueue).change_context(
                    DockerError::ClientError("No tasks available in worker queue".to_string()),
                );

                return Err(error_report);
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
            let error_msg = format!(
                "Invalid transition from {:?} to {:?}",
                persisted.state, task.state
            );

            // Using change_context to add more context to the error
            let error_report = Report::new(WorkerError::InvalidStateTransition(error_msg))
                .change_context(DockerError::ClientError(format!(
                    "State transition validation failed for task {}",
                    task.id
                )));

            return Err(error_report);
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
                let error_report = Report::new(WorkerError::InvalidStateTransition(format!(
                    "Invalid state {:?} for task {}",
                    task.state, task.id
                )))
                .change_context(DockerError::ClientError(format!(
                    "Task {} has invalid state {:?} for execution",
                    task.id, task.state
                )));

                Err(error_report)
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

                // Using change_context to provide more specific context
                let error_report = Report::new(WorkerError::DockerClientError(
                    "Docker client creation failed".to_string(),
                ))
                .change_context(DockerError::ClientError(format!(
                    "Unable to initialize Docker client for task {}",
                    task.id
                )));

                return Err(error_report);
            }
        };

        let result = docker_client.run().await;
        match result {
            Ok(response) => {
                println!(
                    "Task started successfully with container ID: {:?}",
                    response.container_id
                );

                if let Some(container_id) = response.container_id.clone() {
                    task.finish_time = Some(SystemTime::now());
                    task.state = State::Running;
                    task.container_id = Some(container_id);
                    self.db.insert(task.id.clone(), Box::new(task.clone()));
                }
                Ok(response)
            }
            Err(err) => {
                println!("Error running task: {:?}", err);
                task.state = State::Failed;
                Err(err)
            }
        }
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
                // Using change_context for richer context about the stop operation
                let error_report = Report::new(WorkerError::DockerClientError(
                    "Docker client creation failed during stop operation".to_string(),
                ))
                .change_context(DockerError::ClientError(format!(
                    "Unable to initialize Docker client to stop task {}",
                    task.id
                )));

                return Err(error_report);
            }
        };

        let container_id = match task.container_id.clone() {
            Some(id) => id,
            None => {
                println!("No container_id for task");
                // Using error-stack here too for consistency
                let error_report = Report::new(WorkerError::DockerClientError(format!(
                    "Task {} has no container_id for stop operation",
                    task.id
                )))
                .change_context(DockerError::ClientError(format!(
                    "Cannot stop task {} without container_id",
                    task.id
                )));

                return Err(error_report);
            }
        };

        let result = docker_client.stop(&container_id).await;
        match result {
            Ok(response) => {
                task.state = State::Completed;
                task.finish_time = Some(SystemTime::now());

                self.db.insert(task.id.clone(), Box::new(task.clone()));
                println!(
                    "Stopped and removed task with container ID: {:?}",
                    response.container_id
                );

                Ok(response)
            }
            Err(err) => {
                println!("Error stopping task: {:?}", err);
                Err(err)
            }
        }
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.db.values().map(|task| task.as_ref().clone()).collect()
    }
}

pub async fn run_tasks(worker: Arc<Mutex<Worker>>) {
    loop {
        if !worker.lock().await.queue.is_empty() {
            match worker.lock().await.run_task().await {
                Ok(response) => {
                    println!("Task completed successfully: {:?}", response.container_id);
                }
                Err(err) => {
                    println!("Error running task: {:?}", err);
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
        let _stats = get_stats(&worker_guard.sysinfo, worker_guard.task_count);
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub async fn get_system_stats(worker: Arc<Mutex<Worker>>) -> SystemStats {
    let mut worker_guard = worker.lock().await;
    worker_guard.sysinfo.refresh_all();
    get_stats(&worker_guard.sysinfo, worker_guard.task_count)
}
