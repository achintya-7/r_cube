use std::collections::HashMap;
use bollard::{Docker, errors::Error};

#[derive(Debug, Clone)]
pub enum State {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: uuid::Uuid,
    pub name: String,
    pub state: State,
    pub image: String,
    pub memory: u64,
    pub disk: u64,
    pub exposed_ports: Vec<u16>,
    pub port_bindings: HashMap<String, String>,
    pub restart_policy: String,
    pub start_time: Option<std::time::SystemTime>,
    pub finish_time: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct TaskEvent {
    pub task_id: uuid::Uuid,
    pub event_type: String,
    pub timestamp: Option<std::time::SystemTime>,
    pub task: Task,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub name: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub exposed_ports: HashMap<String, HashMap<String, String>>,
    pub cmd: Vec<String>,
    pub image: String,
    pub cpu: f64,
    pub memory: i64,
    pub disk: i64,
    pub env: Vec<String>,
    pub restart_policy: String,
}

#[derive(Debug, Clone)]
pub struct DockerClient {
    pub client: Docker,
    pub config: Config,
}

#[derive(Debug)]
pub struct DockerResult {
    pub error: Option<Error>,
    pub action: Option<String>,
    pub container_id: Option<String>,
    pub result: Option<String>,
}

impl Task {}

