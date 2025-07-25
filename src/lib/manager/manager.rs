use crate::lib::tasks::types::Task;
use crate::lib::{manager::types::Manager, tasks::types::TaskEvent};
use std::io::{Error, ErrorKind};

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
        let mut new_worker = 0;

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
                    println!("Attempting to update task: {}", task.id);

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

    pub fn add_task(&mut self, task_event: TaskEvent) {
        self.pending.push_back(task_event.clone());
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();

        for task in self.task_db.clone().into_iter() {
            tasks.push(task.1);
        }

        tasks
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
