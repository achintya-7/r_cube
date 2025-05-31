pub struct Node {
    pub name: String,
    pub ip: String,
    pub cores: u64,
    pub memory: u64,
    pub memory_allocated: u64,
    pub disk: u64,
    pub disk_allocated: u64,
    pub role: String,
    pub task_count: u64,
}