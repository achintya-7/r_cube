use crate::lib::tasks::types::Task;
use crate::lib::tasks::types::TaskEvent;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Manager {
    pub pending: std::collections::VecDeque<Task>,
    pub task_db: HashMap<String, Task>,
    pub event_db: HashMap<String, TaskEvent>,
    pub workers: Vec<String>,
    pub worker_task_hash_map: HashMap<String, Vec<String>>,
    pub task_worker_hash_map: HashMap<String, String>,
    pub last_worker: u16,
}
