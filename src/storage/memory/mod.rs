use crate::storage::traits::Storage;
use crate::types::{Message, QueueStats, ReapResult};
use async_trait::async_trait;
use base::BaseMemoryStorage;
use std::sync::Arc;
use tokio::sync::Mutex;

mod base;

pub struct MemoryStorage {
    inner: Arc<Mutex<BaseMemoryStorage>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            inner: Arc::new(Mutex::new(BaseMemoryStorage::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn add(&self, msg: Message) -> Result<(), String> {
        let mut storage = self.inner.lock().await;
        storage.add(msg).await
    }

    async fn get(&self, count: usize) -> Result<Vec<Message>, String> {
        let mut storage = self.inner.lock().await;
        storage.get(count).await
    }

    async fn stats(&self) -> Result<QueueStats, String> {
        let storage = self.inner.lock().await;
        storage.stats().await
    }

    async fn delete(&self, ids: Vec<String>) -> Result<(), String> {
        let mut storage = self.inner.lock().await;
        storage.delete(ids).await
    }

    async fn purge(&self) -> Result<(), String> {
        let mut storage = self.inner.lock().await;
        storage.purge().await
    }

    async fn retry(&self, ids: Vec<String>) -> Result<(), String> {
        let mut storage = self.inner.lock().await;
        storage.retry(ids).await
    }

    async fn reap_expired(&self, max_retries: u32) -> Result<ReapResult, String> {
        let (to_retry, to_remove) = {
            let storage = self.inner.lock().await;
            storage.collect_expired(max_retries)
        };

        if to_retry.is_empty() && to_remove.is_empty() {
            return Ok(ReapResult {
                retried: 0,
                dead: 0,
            });
        }

        let mut storage = self.inner.lock().await;
        storage.process_expired(to_retry, to_remove).await
    }
}
