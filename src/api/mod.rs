use crate::services::MessageService;
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

mod handlers;
mod health;
mod models;

pub fn create_api(service: MessageService) -> Router {
    Router::new()
        .route("/hello", get(health::check))
        .route("/stats", get(handlers::stats))
        .route("/add", post(handlers::add_message))
        .route("/get", post(handlers::get_messages))
        .route("/delete", post(handlers::delete_messages))
        .route("/purge", post(handlers::purge_messages))
        .route("/retry", post(handlers::retry_messages))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(service)
}
