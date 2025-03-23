use crate::services::MessageService;
use axum::routing::{delete, get, post};
use axum::Router;

mod handlers;
mod health;
mod models;

pub fn create_api(service: MessageService) -> Router {
    Router::new()
        .route("/hello", get(health::check()))
        .route("/msg", post(handlers::add_message))
        // .route("/messages", get(get_messages))
        // .route("/messages", delete(delete_messages))
        // .route("/messages/purge", post(purge_messages))
        // .route("/messages/retry", post(retry_messages))
        .with_state(service)
}
