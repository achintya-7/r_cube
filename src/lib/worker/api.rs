use axum::{
    Json, Router,
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use uuid::Uuid;

use super::types::{TaskServer, Worker};
use crate::lib::tasks::types::State;
use crate::lib::tasks::types::{Task, TaskEvent};
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
        let server = server.lock().await;
        let worker = server.worker.lock().await;
        Json(worker.get_tasks())
    }

    async fn start_task(
        AxumState(server): AxumState<Arc<Mutex<TaskServer>>>,
        Json(task_event): Json<TaskEvent>,
    ) -> impl IntoResponse {
        let server = server.lock().await;
        server.worker.lock().await.add_task(task_event.task.clone());
        println!("Task Queued to start: {:?}", task_event.task_id);
        StatusCode::CREATED
    }

    async fn stop_task(
        AxumState(server): AxumState<Arc<Mutex<TaskServer>>>,
        Path(id): Path<Uuid>,
    ) -> impl IntoResponse {
        let server = server.lock().await;

        let mut task_to_stop = match server.worker.lock().await.db.get(&id) {
            Some(task) => task.clone(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("Task with id {} not found", id),
                );
            }
        };

        task_to_stop.state = State::Completed;

        server.worker.lock().await.add_task(*task_to_stop);

        println!("Task stopped: {:?}", id);
        (StatusCode::OK, format!("Task with id {} stopped", id))
    }

    pub async fn start_server(self) {
        let address = self.address.clone();
        let port = self.port.clone();

        let shared_server = Arc::new(Mutex::new(self));

        println!("Starting TaskServer at {}:{}", address, port);
        let app = Router::new()
            .route("/tasks", get(TaskServer::get_tasks))
            .route("/tasks", post(TaskServer::start_task))
            .route("/tasks/{id}", delete(TaskServer::stop_task))
            .with_state(shared_server);

        // Start the server
        println!("Listening on {}:{}", address, port);
        let listener = TcpListener::bind(format!("{}:{}", address, port))
            .await
            .unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
