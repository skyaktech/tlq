use crate::types::{Message, MessageState};
use std::collections::HashMap;

pub struct BaseMemoryStorage {
    queue: Vec<Message>,
    processing: HashMap<String, Message>,
}

impl BaseMemoryStorage {
    pub(crate) fn new() -> Self {
        BaseMemoryStorage {
            queue: Vec::new(),
            processing: HashMap::new(),
        }
    }

    pub(crate) async fn add(&mut self, msg: Message) -> Result<(), String> {
        self.queue.push(msg);
        Ok(())
    }

    pub(crate) async fn get(&mut self, count: usize) -> Result<Vec<Message>, String> {
        let count = count.min(self.queue.len());
        let mut messages: Vec<Message> = self.queue.drain(0..count).collect();
        for message in &mut messages {
            message.state = MessageState::Processing;

            self.processing
                .insert(message.id.to_string(), message.clone());
        }

        Ok(messages)
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
        Ok(())
    }

    pub(crate) async fn retry(&mut self, ids: Vec<String>) -> Result<(), String> {
        let mut retried_messages = Vec::new();

        for id in &ids {
            if let Some(mut message) = self.processing.remove(id) {
                message.retry_count += 1;
                message.state = MessageState::Ready;

                retried_messages.push(message);
            }
        }

        self.queue.extend(retried_messages);

        Ok(())
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
        }
    }

    #[tokio::test]
    async fn test_new_base_memory_storage() {
        let storage = BaseMemoryStorage::new();
        assert_eq!(storage.queue.len(), 0);
        assert_eq!(storage.processing.len(), 0);
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
}
