use crate::storage::traits::Storage;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

pub async fn start_reaper(storage: Arc<dyn Storage>, max_retries: u32, interval_secs: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        match storage.reap_expired(max_retries).await {
            Ok(result) if result.retried > 0 || result.dead > 0 => {
                info!(
                    "Reaper: retried={}, removed={}",
                    result.retried, result.dead
                );
            }
            Err(e) => warn!("Reaper error: {}", e),
            _ => {}
        }
    }
}
