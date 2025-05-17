pub struct worker {
    pub name: String,
    pub queue: std::collections::VecDeque<task::Task>,
    pub db: Map<uuid::Uuid, Box<task::Task>>,
    pub task_count: u64,
}

