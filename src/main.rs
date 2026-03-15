use std::sync::Arc;
use tlq::api::create_api;
use tlq::config::config;
use tlq::services::MessageService;
use tlq::storage::memory::MemoryStorage;
use tlq::storage::traits::Storage;
use tracing::info;
use tracing_subscriber::{
    filter::LevelFilter, layer::Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() {
    let cfg = config();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(LevelFilter::from_level(cfg.tracing_level())),
        )
        .init();

    info!(
        "Starting TLQ with configuration: port={}, max_message_size={}, log_level={}, lock_duration={}s, max_retries={}",
        cfg.port, cfg.max_message_size, cfg.log_level, cfg.lock_duration_secs, cfg.max_retries
    );

    let store = Arc::new(MemoryStorage::new());
    let reaper_store: Arc<dyn Storage> = store.clone();
    let service = MessageService::new(store);

    tokio::spawn(tlq::worker::start_reaper(
        reaper_store,
        cfg.max_retries,
        cfg.worker_interval_secs,
    ));

    let app = create_api(service);
    let bind_addr = format!("[::]:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
