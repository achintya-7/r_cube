use super::types::{Worker, valid_state_transition};
use crate::lib::tasks::types::{DockerClient, DockerResult, State, Task, new_config};
use std::{io::Error, io::ErrorKind::Other, time::SystemTime};

impl Worker {
    pub fn collect_stats(&self) {
        println!("I will collect stats");
    }

    pub async fn run_task(&mut self) -> DockerResult {
        let task_queued = match self.queue.pop_front() {
            Some(task) => task,
            None => {
                println!("No tasks in queue");
                return DockerResult::with_error(Box::new(Error::new(Other, "No tasks in queue")));
            }
        };

        let task_persisted = match self.db.get(&task_queued.id) {
            Some(task) => task,
            None => {
                self.db
                    .insert(task_queued.id, Box::new(task_queued.clone()));
                &task_queued
            }
        };

        if !valid_state_transition(&task_persisted.state, &task_queued.state) {
            println!(
                "Invalid state transition from {:?} to {:?}",
                task_persisted.state, task_queued.state
            );
            return DockerResult::with_error(Box::new(Error::new(
                Other,
                "Invalid state transition",
            )));
        }

        match task_queued.state {
            State::Scheduled => {
                println!("Task is scheduled, starting it now");
                return self.start_task(task_queued).await;
            }

            State::Completed => {
                println!("Task is already running, stopping it now");
                return self.stop_task(task_queued).await;
            }

            _ => {
                println!("Invalid state for task: {:?} with id: {:?}", task_queued.state, task_queued.id);
                return DockerResult::with_error(Box::new(Error::new(
                    Other,
                    "Invalid state for task",
                )));
            }
        }
    }

    pub async fn start_task(&mut self, mut task: Task) -> DockerResult {
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
        match &result.error {
            Some(err) => {
                println!("Error running task: {:?}", err);
                task.state = State::Failed;
                return result;
            }
            None => {
                println!(
                    "Task started successfully with container ID: {:?}",
                    result.container_id
                );

                let container_id = result.container_id.clone().unwrap();

                task.finish_time = Some(SystemTime::now());
                task.state = State::Running;
                task.container_id = Some(container_id.clone());

                self.db.insert(task.id, Box::new(task.clone()));

                return result;
            }
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.queue.push_back(task.clone());
    }

    pub async fn stop_task(&mut self, mut task: Task) -> DockerResult {
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

        let container_id = task.container_id.clone().unwrap();

        let result = docker_client.stop(&container_id).await;
        match &result.error {
            Some(err) => {
                println!("Error stopping task: {:?}", err);
                return result;
            }
            None => {
                task.state = State::Completed;
                task.finish_time = Some(SystemTime::now());

                self.db.insert(task.id, Box::new(task.clone()));

                println!(
                    "Stopped and removed task with container ID: {:?}",
                    result.container_id
                );

                return result;
            }
        }
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        return self.db.values().cloned().map(|task| *task).collect();
    }
}
