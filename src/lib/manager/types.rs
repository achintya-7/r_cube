use tokio::sync::Mutex;

use crate::lib::tasks::types::Task;
use crate::lib::tasks::types::TaskEvent;
use std::collections::HashMap;

use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Manager {
    pub pending: std::collections::VecDeque<TaskEvent>,
    pub task_db: HashMap<String, Task>,
    pub event_db: HashMap<String, TaskEvent>,
    pub workers: Vec<String>,
    pub worker_task_hash_map: HashMap<String, Vec<String>>,
    pub task_worker_hash_map: HashMap<String, String>,
    pub last_worker: u16,
}

pub struct ManagerServer {
    pub address: String,
    pub port: String,
    pub manager: Arc<Mutex<Manager>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManagerError {
    NoWorkersAvailable,
    WorkerCommunication(String),
    NetworkError(String),
}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagerError::NoWorkersAvailable => {
                write!(f, "No workers are available to handle tasks")
            }
            ManagerError::WorkerCommunication(msg) => {
                write!(f, "Worker communication failed: {}", msg)
            }
            ManagerError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
        }
    }
}

impl Error for ManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type ManagerResult<T> = Result<T, ManagerError>;
