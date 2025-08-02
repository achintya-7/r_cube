use serde::Deserialize;

use tokio::sync::Mutex;

use crate::lib::tasks::types::{DockerError, Task};
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

pub struct Worker {
    pub name: String,
    pub queue: std::collections::VecDeque<Task>,
    pub db: HashMap<String, Box<Task>>,
    pub task_count: u64,
    pub sysinfo: sysinfo::System,
}

#[derive(Deserialize, Debug)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub system_name: String,
    pub hostname: String,
    pub total_cpus: u64,
    pub disk_usage: f32,
    pub task_count: u64,
}

pub struct TaskServer {
    pub worker: Arc<Mutex<Worker>>,
    pub address: String,
    pub port: String,
}

#[derive(Debug, Clone)]
pub enum WorkerError {
    NoTasksInQueue,
    InvalidStateTransition(String),
    DockerClientError(String),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerError::NoTasksInQueue => write!(f, "No tasks in the queue"),
            WorkerError::InvalidStateTransition(msg) => {
                write!(f, "Invalid state transition: {}", msg)
            }
            WorkerError::DockerClientError(msg) => {
                write!(f, "Docker client error: {}", msg)
            }
        }
    }
}

impl Error for WorkerError {}

impl From<WorkerError> for DockerError {
    fn from(worker_error: WorkerError) -> Self {
        match worker_error {
            WorkerError::NoTasksInQueue => {
                DockerError::ClientError("No tasks in queue".to_string())
            }
            WorkerError::InvalidStateTransition(msg) => {
                DockerError::ClientError(format!("Invalid state transition: {}", msg))
            }
            WorkerError::DockerClientError(msg) => DockerError::ClientError(msg),
        }
    }
}

pub type WorkerResult<T> = Result<T, WorkerError>;
