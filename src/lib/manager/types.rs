use std::collections::HashMap;
use crate::lib::tasks::types::Task;
use crate::lib::tasks::types::TaskEvent;

pub struct Manager {
    pub pending: std::collections::VecDeque<Task>,
    pub task_db: HashMap<uuid::Uuid, Task>,
    pub event_db: HashMap<uuid::Uuid, TaskEvent>,
    pub workers: Vec<String>,
    pub worker_task_hash_map: HashMap<String, Vec<uuid::Uuid>>,
    pub task_worker_hash_map: HashMap<uuid::Uuid, String>,
}