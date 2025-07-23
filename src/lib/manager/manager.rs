use crate::lib::tasks::types::Task;
use crate::lib::{manager::types::Manager, tasks::types::TaskEvent};
use std::io::{Error, ErrorKind};
use std::time::SystemTime;

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

    pub fn select_worker(&mut self) -> String {
        let mut new_worker: u16 = 0;
        if self.last_worker + 1 < self.workers.len() as u16 {
            new_worker = self.last_worker + 1;
        } else {
            new_worker = 0;
            self.last_worker = 0;
        }

        return self.workers[new_worker as usize].clone();
    }

    pub async fn update_task(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for worker in &self.workers {
            println!("Checking worker: {}", worker);

            let tasks = self.get_worker_tasks(worker.clone()).await?;
            for task in tasks {
                if let Some(_) = self.event_db.get(&task.id) {
                    println!("Attepting to update task: {}", task.id);

                    if self.task_db.contains_key(&task.id) {
                        let local_task = self.task_db.get(&task.id).unwrap();

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

        Ok(())
    }

    pub fn add_task(&mut self, task: Task) {
        self.pending.push_back(task.clone());
    }

    async fn get_worker_tasks(
        &self,
        worker: String,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let url = format!("http://{}/tasks/", worker);

        let client = reqwest::Client::new();
        let resp = client.get(&url).send().await?;

        if resp.status().is_success() {
            let tasks: Vec<Task> = resp.json().await?;
            println!("Tasks from worker {}: {:?}", worker, tasks);
            Ok(tasks)
        } else {
            Err(Box::new(Error::new(
                ErrorKind::Other,
                format!(
                    "Failed to fetch tasks from worker {}: {:?}",
                    worker,
                    resp.status()
                ),
            )))
        }
    }

    async fn send_worker_event(
        &self,
        worker: String,
        task_event: TaskEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("http://{}/tasks", worker);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&task_event)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Failed to send event: {:?}", response.status()),
            )))
        }
    }

    pub async fn send_work(&mut self) {
        if !self.pending.is_empty() {
            let worker = self.select_worker();

            let event = self.pending.pop_back().unwrap();
            let task_event = TaskEvent {
                task_id: event.id.clone(),
                event_type: "scheduled".to_string(),
                timestamp: Some(SystemTime::now()),
                task: event.clone(),
            };

            self.event_db
                .insert(event.id.clone().to_string(), task_event.clone());

            self.worker_task_hash_map
                .entry(worker.clone())
                .or_default()
                .push(event.id.clone());

            self.task_worker_hash_map
                .insert(event.id.clone(), worker.clone());

            self.task_db.insert(event.id.clone(), event);

            let resp = self.send_worker_event(worker, task_event).await;
            match resp {
                Ok(_) => {
                    println!("Event sent successfully");
                }
                Err(e) => {
                    eprintln!("Error sending event: {:?}", e);
                }
            }
        } else {
            println!("No pending tasks to send");
        }
    }
}
