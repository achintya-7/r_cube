use serde::Deserialize;

use tokio::sync::Mutex;

use crate::lib::tasks::types::{Task};
use std::{collections::HashMap, sync::Arc};

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
