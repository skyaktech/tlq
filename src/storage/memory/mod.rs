use crate::types::Message;
use async_trait::async_trait;
use base::BaseMemoryStorage;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::storage::traits::Storage;

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
}
