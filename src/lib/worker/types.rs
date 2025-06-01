use tokio::sync::Mutex;

use crate::lib::tasks::types::{State, Task};
use std::{collections::HashMap, sync::Arc};

pub struct Worker {
    pub name: String,
    pub queue: std::collections::VecDeque<Task>,
    pub db: HashMap<uuid::Uuid, Box<Task>>,
    pub task_count: u64,
}

pub struct TaskServer {
    pub worker: Arc<Mutex<Worker>>,
    pub address: String,
    pub port: String,
}
