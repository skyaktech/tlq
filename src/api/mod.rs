use crate::services::MessageService;
use axum::routing::{get, post};
use axum::Router;

mod handlers;
mod health;
mod models;

pub fn create_api(service: MessageService) -> Router {
    Router::new()
        .route("/hello", get(health::check()))
        .route("/msg/add", post(handlers::add_message))
        .route("/msg/get", post(handlers::get_messages))
        // .route("/msg/delete", delete(delete_messages))
        // .route("/msg/purge", post(purge_messages))
        // .route("/msg/retry", post(retry_messages))
        .with_state(service)
}
