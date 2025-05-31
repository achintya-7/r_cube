use crate::lib::tasks::types::Task;
use std::collections::HashMap;

pub struct Worker {
    pub name: String,
    pub queue: std::collections::VecDeque<Task>,
    pub db: HashMap<uuid::Uuid, Box<Task>>,
    pub task_count: u64,
}
