pub enum State {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
}

pub struct Task {
    pub id: uuid::Uuid,
    pub name: String,
    pub state: State,
    pub image: String,
    pub memory: u64,
    pub disk: u64,
    pub exposed_ports: Vec<u16>,
    pub port_bindings: Map<String, String>,
    pub restart_policy: String,
    pub start_time: std::time::SystemTime,
    pub finish_time: std::time::SystemTime,
}

pub struct TaskEvent {
    pub task_id: uuid::Uuid,
    pub event_type: String,
    pub timestamp: std::time::SystemTime,
    pub task: Task,
}

impl Task {

}