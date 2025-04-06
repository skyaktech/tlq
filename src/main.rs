use std::sync::Arc;
use tlq::api::create_api;
use tlq::services::MessageService;
use tlq::storage::memory::MemoryStorage;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let store = Arc::new(MemoryStorage::new());
    let service = MessageService::new(store);

    let app = create_api(service);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
