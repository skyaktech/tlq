use crate::config;
use crate::types::{Message, MessageState, QueueStats, ReapResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub struct BaseMemoryStorage {
    queue: Vec<Message>,
    processing: HashMap<String, Message>,
    dead_count: usize,
}

impl BaseMemoryStorage {
    pub(crate) fn new() -> Self {
        BaseMemoryStorage {
            queue: Vec::new(),
            processing: HashMap::new(),
            dead_count: 0,
        }
    }

    pub(crate) async fn add(&mut self, msg: Message) -> Result<(), String> {
        self.queue.push(msg);
        Ok(())
    }

    pub(crate) async fn get(&mut self, count: usize) -> Result<Vec<Message>, String> {
        let count = count.min(self.queue.len());
        let lock_until = now_millis() + (config::config().lock_duration_secs * 1000) as i64;
        let mut messages: Vec<Message> = self.queue.drain(0..count).collect();
        for message in &mut messages {
            message.state = MessageState::Processing;
            message.lock_until = Some(lock_until);

            self.processing
                .insert(message.id.to_string(), message.clone());
        }

        Ok(messages)
    }

    pub(crate) async fn stats(&self) -> Result<QueueStats, String> {
        Ok(QueueStats {
            ready: self.queue.len(),
            processing: self.processing.len(),
            dead: self.dead_count,
        })
    }

    pub(crate) async fn delete(&mut self, ids: Vec<String>) -> Result<(), String> {
        for id in ids {
            self.processing.remove(&id);
        }
        Ok(())
    }

    pub(crate) async fn purge(&mut self) -> Result<(), String> {
        self.queue.clear();
        self.processing.clear();
        self.dead_count = 0;
        Ok(())
    }

    pub(crate) async fn retry(&mut self, ids: Vec<String>) -> Result<(), String> {
        let mut retried_messages = Vec::new();

        for id in &ids {
            if let Some(mut message) = self.processing.remove(id) {
                message.retry_count += 1;
                message.state = MessageState::Ready;
                message.lock_until = None;

                retried_messages.push(message);
            }
        }

        self.queue.extend(retried_messages);

        Ok(())
    }

    pub(crate) fn collect_expired(&self, max_retries: u32) -> (Vec<String>, Vec<String>) {
        let now_ms = now_millis();
        let mut to_retry = Vec::new();
        let mut to_remove = Vec::new();

        for (id, msg) in &self.processing {
            if let Some(lock_until) = msg.lock_until {
                if now_ms >= lock_until {
                    if (msg.retry_count as u32) < max_retries {
                        to_retry.push(id.clone());
                    } else {
                        to_remove.push(id.clone());
                    }
                }
            }
        }

        (to_retry, to_remove)
    }

    pub(crate) async fn process_expired(
        &mut self,
        to_retry: Vec<String>,
        to_remove: Vec<String>,
    ) -> Result<ReapResult, String> {
        let retried = to_retry.len();
        let dead = to_remove.len();
        self.retry(to_retry).await?;
        self.delete(to_remove).await?;
        self.dead_count += dead;
        Ok(ReapResult { retried, dead })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_storage() -> BaseMemoryStorage {
        BaseMemoryStorage {
            queue: vec![
                Message::new("Hello World".to_string()),
                Message::new("Hello Solar System".to_string()),
                Message::new("Hello Universe".to_string()),
            ],
            processing: HashMap::new(),
            dead_count: 0,
        }
    }

    #[tokio::test]
    async fn test_new_base_memory_storage() {
        let storage = BaseMemoryStorage::new();
        assert_eq!(storage.queue.len(), 0);
        assert_eq!(storage.processing.len(), 0);
        assert_eq!(storage.dead_count, 0);
    }

    #[tokio::test]
    async fn test_base_memory_storage_add() {
        let mut storage = setup_storage();

        let msg = Message::new("Hello Serbia".to_string());
        storage.add(msg).await.unwrap();
        let msg = Message::new("Hello Balkan".to_string());
        storage.add(msg).await.unwrap();

        assert_eq!(storage.queue.len(), 5);
    }

    #[tokio::test]
    async fn test_base_memory_storage_get() {
        let mut storage = setup_storage();

        let messages = storage.get(2).await.unwrap();
        assert_eq!(messages.len(), 2);
        for message in &messages {
            assert_eq!(message.state, MessageState::Processing);
        }
        assert_eq!(storage.queue.len(), 1);
        assert_eq!(storage.processing.len(), 2);
    }

    #[tokio::test]
    async fn test_base_memory_storage_get_more_than_available() {
        let mut storage = setup_storage();

        let messages = storage.get(5).await.unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(storage.queue.len(), 0);
        assert_eq!(storage.processing.len(), 3);
    }

    #[tokio::test]
    async fn test_base_memory_storage_delete() {
        let mut storage = setup_storage();

        let messages = storage.get(2).await.unwrap();
        storage
            .delete(vec![messages[0].id.to_string(), messages[1].id.to_string()])
            .await
            .unwrap();
        assert_eq!(storage.processing.len(), 0);
    }

    #[tokio::test]
    async fn test_base_memory_storage_delete_non_existent() {
        let mut storage = setup_storage();

        let _messages = storage.get(2).await.unwrap();
        storage
            .delete(vec!["non-existent-id".to_string()])
            .await
            .unwrap();
        assert_eq!(storage.processing.len(), 2);
    }

    #[tokio::test]
    async fn test_base_memory_storage_delete_duplicate() {
        let mut storage = setup_storage();

        let messages = storage.get(2).await.unwrap();
        storage
            .delete(vec![messages[0].id.to_string(), messages[0].id.to_string()])
            .await
            .unwrap();
        assert_eq!(storage.processing.len(), 1);
    }

    #[tokio::test]
    async fn test_base_memory_storage_purge() {
        let mut storage = setup_storage();
        let _messages = storage.get(1).await.unwrap();

        storage.purge().await.unwrap();
        assert_eq!(storage.queue.len(), 0);
        assert_eq!(storage.processing.len(), 0);
    }

    #[tokio::test]
    async fn test_base_memory_storage_retry() {
        let mut storage = setup_storage();
        let messages = storage.get(2).await.unwrap();
        storage
            .retry(vec![messages[0].id.to_string()])
            .await
            .unwrap();

        assert_eq!(storage.queue.len(), 2);
        assert_eq!(storage.processing.len(), 1);
    }

    #[tokio::test]
    async fn test_base_memory_storage_stats() {
        let mut storage = setup_storage();

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.ready, 3);
        assert_eq!(stats.processing, 0);

        storage.get(2).await.unwrap();

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.ready, 1);
        assert_eq!(stats.processing, 2);
    }

    #[tokio::test]
    async fn test_get_sets_lock_until() {
        let mut storage = setup_storage();
        let messages = storage.get(2).await.unwrap();

        for msg in &messages {
            assert!(msg.lock_until.is_some());
        }
        for msg in storage.processing.values() {
            assert!(msg.lock_until.is_some());
        }
    }

    #[tokio::test]
    async fn test_retry_clears_lock_until() {
        let mut storage = setup_storage();
        let messages = storage.get(1).await.unwrap();
        let id = messages[0].id.to_string();

        storage.retry(vec![id]).await.unwrap();

        let retried = &storage.queue.last().unwrap();
        assert!(retried.lock_until.is_none());
        assert_eq!(retried.state, MessageState::Ready);
    }

    fn insert_processing(storage: &mut BaseMemoryStorage, lock_until: i64, retry_count: i32) {
        let mut msg = Message::new("test".to_string());
        msg.state = MessageState::Processing;
        msg.lock_until = Some(lock_until);
        msg.retry_count = retry_count;
        storage.processing.insert(msg.id.to_string(), msg);
    }

    #[tokio::test]
    async fn test_collect_expired_retries_under_max() {
        let mut storage = BaseMemoryStorage::new();
        insert_processing(&mut storage, 0, 0); // expired, under max

        let (to_retry, to_remove) = storage.collect_expired(3);
        assert_eq!(to_retry.len(), 1);
        assert_eq!(to_remove.len(), 0);
    }

    #[tokio::test]
    async fn test_collect_expired_removes_at_max() {
        let mut storage = BaseMemoryStorage::new();
        insert_processing(&mut storage, 0, 3); // expired, at max

        let (to_retry, to_remove) = storage.collect_expired(3);
        assert_eq!(to_retry.len(), 0);
        assert_eq!(to_remove.len(), 1);
    }

    #[tokio::test]
    async fn test_collect_expired_ignores_unexpired() {
        let mut storage = BaseMemoryStorage::new();
        insert_processing(&mut storage, i64::MAX, 0); // not expired

        let (to_retry, to_remove) = storage.collect_expired(3);
        assert_eq!(to_retry.len(), 0);
        assert_eq!(to_remove.len(), 0);
    }

    #[tokio::test]
    async fn test_collect_expired_mixed() {
        let mut storage = BaseMemoryStorage::new();
        insert_processing(&mut storage, 0, 0); // expired, retry
        insert_processing(&mut storage, 0, 3); // expired, dead
        insert_processing(&mut storage, i64::MAX, 0); // not expired

        let (to_retry, to_remove) = storage.collect_expired(3);
        assert_eq!(to_retry.len(), 1);
        assert_eq!(to_remove.len(), 1);
        assert_eq!(storage.processing.len(), 3);
    }

    #[tokio::test]
    async fn test_dead_count_in_stats() {
        let mut storage = BaseMemoryStorage::new();
        storage.dead_count = 3;

        let stats = storage.stats().await.unwrap();
        assert_eq!(stats.dead, 3);
    }

    #[tokio::test]
    async fn test_purge_resets_dead_count() {
        let mut storage = BaseMemoryStorage::new();
        storage.dead_count = 5;
        storage.purge().await.unwrap();
        assert_eq!(storage.dead_count, 0);
    }
}
