pub struct Manager {
    pub pending: std::collections::VecDeque<task::Task>,
    pub task_db: Map<uuid::Uuid, task::Task>,
    pub event_db: Map<uuid::Uuid, task::TaskEvent>,
    pub workers: Vec<String>,
    pub worker_task_map: Map<String, Vec<uuid::Uuid>>,
    pub task_worker_map: Map<uuid::Uuid, String>,
}