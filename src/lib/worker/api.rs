use axum::{
    Json, Router,
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};

use super::types::{TaskServer, Worker};
use crate::lib::tasks::types::{Task, TaskEvent};
use crate::lib::{
    tasks::types::State,
    worker::{stats::get_stats, types::SystemStats},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

impl TaskServer {
    pub fn new(worker: Arc<Mutex<Worker>>, address: &str, port: &str) -> Self {
        Self {
            worker,
            address: address.to_string(),
            port: port.to_string(),
        }
    }

    async fn get_tasks(AxumState(server): AxumState<Arc<Mutex<TaskServer>>>) -> Json<Vec<Task>> {
        let worker = server.lock().await.worker.clone();
        let tasks = worker.lock().await.get_tasks();
        Json(tasks)
    }

    async fn start_task(
        AxumState(server): AxumState<Arc<Mutex<TaskServer>>>,
        Json(task_event): Json<TaskEvent>,
    ) -> impl IntoResponse {
        let worker = server.lock().await.worker.clone();
        worker.lock().await.add_task(task_event.task.clone());
        println!("Task Queued to start: {:?}", task_event.task_id);
        StatusCode::CREATED
    }

    async fn stop_task(
        AxumState(server): AxumState<Arc<Mutex<TaskServer>>>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let worker = server.lock().await.worker.clone();
        let mut guard = worker.lock().await;
        let task = match guard.db.get(&id) {
            Some(task) => task.as_ref().clone(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("Task with id {} not found", id),
                );
            }
        };

        let mut stopped_task = task;
        stopped_task.state = State::Completed;
        guard.add_task(stopped_task);
        println!("Task stopped: {:?}", id);
        (StatusCode::OK, format!("Task with id {} stopped", id))
    }

    pub async fn get_stats(
        AxumState(server): AxumState<Arc<Mutex<TaskServer>>>,
    ) -> impl IntoResponse {
        let worker_guard = server.lock().await.worker.clone();
        let mut worker_guard = worker_guard.lock().await;
        worker_guard.sysinfo.refresh_all();
        let stats = get_stats(&worker_guard.sysinfo, worker_guard.task_count);
        (StatusCode::OK, Json(stats))
    }

    pub async fn start_server(self) {
        let address = self.address.clone();
        let port = self.port.clone();
        let shared = Arc::new(Mutex::new(self));
        println!("Starting TaskServer at {}:{}", address, port);

        let app = Router::new()
            .route("/stats", get(TaskServer::get_stats))
            .route("/tasks", get(TaskServer::get_tasks))
            .route("/tasks", post(TaskServer::start_task))
            .route("/tasks/{id}", delete(TaskServer::stop_task))
            .with_state(shared);

        println!("Listening on {}:{}", address, port);
        let listener = TcpListener::bind(format!("{}:{}", address, port))
            .await
            .unwrap();

        axum::serve(listener, app).await.unwrap();
    }
}
