use crate::services::MessageService;
use axum::routing::{get, post};
use axum::Router;

mod handlers;
mod health;
mod models;

pub fn create_api(service: MessageService) -> Router {
    Router::new()
        .route("/hello", get(health::check))
        .route("/add", post(handlers::add_message))
        .route("/get", post(handlers::get_messages))
        .route("/delete", post(handlers::delete_messages))
        .route("/purge", post(handlers::purge_messages))
        .route("/retry", post(handlers::retry_messages))
        .with_state(service)
}
