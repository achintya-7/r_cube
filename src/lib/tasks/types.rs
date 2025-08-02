use std::{collections::HashMap, error::Error, fmt};

use bollard::Docker;
use error_stack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub container_id: Option<String>,
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

impl Default for Task {
    fn default() -> Self {
        Task {
            id: uuid::Uuid::new_v4().to_string(),
            container_id: None,
            name: String::new(),
            state: State::Pending,
            image: String::new(),
            memory: 0,
            disk: 0,
            exposed_ports: Vec::new(),
            port_bindings: HashMap::new(),
            restart_policy: String::new(),
            start_time: None,
            finish_time: None,
        }
    }
}

impl Task {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub task_id: String,
    pub event_type: String,
    pub timestamp: Option<std::time::SystemTime>,
    pub task: Task,
}

impl Default for TaskEvent {
    fn default() -> Self {
        TaskEvent {
            task_id: String::new(),
            event_type: String::new(),
            timestamp: None,
            task: Task::default(),
        }
    }
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

pub fn new_config(task: Task) -> Config {
    return Config {
        name: task.name,
        image: task.image,
        restart_policy: task.restart_policy,
        ..Default::default()
    };
}

#[derive(Debug, Clone)]
pub struct DockerClient {
    pub client: Docker,
    pub config: Config,
}

// * DockerResponse is a simplified response type for Docker operations
#[derive(Debug)]
pub struct DockerResponse {
    pub error: Option<DockerError>,
    pub action: Option<String>,
    pub container_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DockerError {
    ClientError(String),
    ImagePullError(String),
    ContainerCreationError(String),
    ContainerStartError(String),
    ContainerStopError(String),
}

impl fmt::Display for DockerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockerError::ClientError(msg) => write!(f, "Docker client error: {}", msg),
            DockerError::ImagePullError(msg) => write!(f, "Docker image pull error: {}", msg),
            DockerError::ContainerCreationError(msg) => {
                write!(f, "Container creation error: {}", msg)
            }
            DockerError::ContainerStartError(msg) => write!(f, "Container start error: {}", msg),
            DockerError::ContainerStopError(msg) => write!(f, "Container stop error: {}", msg),
        }
    }
}

impl Error for DockerError {}

pub type DockerResult = Result<DockerResponse, error_stack::Report<DockerError>>;
