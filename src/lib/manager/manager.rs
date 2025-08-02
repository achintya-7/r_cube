use crate::lib::manager::types::{ManagerError, ManagerResult};
use crate::lib::tasks::types::Task;
use crate::lib::{manager::types::Manager, tasks::types::TaskEvent};

impl Manager {
    pub fn new(workers: Vec<String>) -> Self {
        Manager {
            workers,
            last_worker: 0,
            pending: std::collections::VecDeque::new(),
            task_db: std::collections::HashMap::new(),
            event_db: std::collections::HashMap::new(),
            worker_task_hash_map: std::collections::HashMap::new(),
            task_worker_hash_map: std::collections::HashMap::new(),
        }
    }

    pub fn select_worker(&mut self) -> ManagerResult<String> {
        if self.workers.is_empty() {
            return Err(ManagerError::NoWorkersAvailable);
        }

        let new_worker = if self.last_worker + 1 < self.workers.len() as u16 {
            self.last_worker + 1
        } else {
            self.last_worker = 0;
            0
        };

        self.last_worker = new_worker;
        Ok(self.workers[new_worker as usize].clone())
    }

    pub async fn update_task(&mut self) -> ManagerResult<()> {
        for worker in &self.workers {
            println!("Checking worker: {}", worker);

            let tasks = self.get_worker_tasks(worker.clone()).await?;
            for task in tasks {
                if let Some(_) = self.event_db.get(&task.id) {
                    println!("Attempting to update task: {}", task.id);

                    if self.task_db.contains_key(&task.id) {
                        if let Some(local_task) = self.task_db.get(&task.id) {
                            let new_task = Task {
                                container_id: task.container_id.clone(),
                                start_time: task.start_time,
                                finish_time: task.finish_time,
                                state: task.state.clone(),
                                ..local_task.clone()
                            };

                            if local_task.state != task.state {
                                self.task_db.insert(task.id.clone(), new_task);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_task(&mut self, task_event: TaskEvent) {
        self.pending.push_back(task_event.clone());
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        self.task_db.values().cloned().collect()
    }

    async fn get_worker_tasks(&self, worker: String) -> ManagerResult<Vec<Task>> {
        let url = format!("http://{}/tasks/", worker);

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|_| ManagerError::NetworkError(format!("Failed to connect to {}", url)))?;

        if resp.status().is_success() {
            let tasks: Vec<Task> = resp.json().await.map_err(|_| {
                ManagerError::WorkerCommunication(format!(
                    "Failed to parse response from worker {}",
                    worker
                ))
            })?;
            println!("Tasks from worker {}: {:?}", worker, tasks);
            Ok(tasks)
        } else {
            Err(ManagerError::WorkerCommunication(format!(
                "Worker {} returned status {}",
                worker,
                resp.status().as_u16()
            )))
        }
    }

    async fn send_worker_event(&self, worker: String, task_event: TaskEvent) -> ManagerResult<()> {
        let url = format!("http://{}/tasks", worker);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&task_event)
            .send()
            .await
            .map_err(|_| ManagerError::NetworkError(format!("Failed to connect to {}", url)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ManagerError::WorkerCommunication(format!(
                "Failed to send task {} to worker {}",
                task_event.task_id, worker
            )))
        }
    }

    pub async fn send_work(&mut self) -> ManagerResult<()> {
        if !self.pending.is_empty() {
            let worker = self.select_worker()?;

            let task_event = self.pending.pop_back().unwrap();

            self.event_db
                .insert(task_event.task_id.clone(), task_event.clone());

            self.worker_task_hash_map
                .entry(worker.clone())
                .or_default()
                .push(task_event.task_id.clone());

            self.task_worker_hash_map
                .insert(task_event.task_id.clone(), worker.clone());

            self.task_db
                .insert(task_event.task_id.clone(), task_event.task.clone());

            match self.send_worker_event(worker, task_event).await {
                Ok(_) => {
                    println!("Event sent successfully");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error sending event: {:?}", e);
                    Err(e)
                }
            }
        } else {
            println!("No pending tasks to send");
            Ok(())
        }
    }
}
