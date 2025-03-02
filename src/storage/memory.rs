use std::collections::HashSet;
use crate::types::{Message, Storage};
use async_trait::async_trait;

pub struct InMemoryStorage {
    queue: Vec<Message>,
    in_flight: Vec<Message>,
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn add(&mut self, msg: Message) -> Result<(), String> {
        self.queue.push(msg);
        Ok(())
    }

    async fn get(&mut self, count: usize) -> Result<Vec<Message>, String> {
        let count = count.min(self.queue.len());
        let messages: Vec<Message> = self.queue.drain(0..count).collect();
        self.in_flight.extend(messages.clone());
        Ok(messages)
    }

    async fn delete(&mut self, ids: Vec<String>) -> Result<(), String> {
        let id_set : HashSet<&String> = ids.iter().collect();
        self.in_flight.retain(|msg| !id_set.contains(&msg.id.to_string()));
        Ok(())
    }

    async fn purge(&mut self) -> Result<(), String> {
        // remove all messages from in_flight and queue
        todo!()
    }

    async fn retry(&mut self, _id: String) -> Result<(), String> {
        // move message from in_flight back to queue, increment retry_count
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_storage() -> InMemoryStorage {
        InMemoryStorage {
            queue: vec![
                Message::new("Hello World".to_string()),
                Message::new("Hello Solar System".to_string()),
                Message::new("Hello Universe".to_string()),
            ],
            in_flight: vec![],
        }
    }

    #[tokio::test]
    async fn test_in_memory_storage_add() {
        let mut storage = setup_storage();

        let msg = Message::new("Hello Serbia".to_string());
        storage.add(msg).await.unwrap();
        let msg = Message::new("Hello Balkan".to_string());
        storage.add(msg).await.unwrap();

        assert_eq!(storage.queue.len(), 5);
    }

    #[tokio::test]
    async fn test_in_memory_storage_get() {
        let mut storage = setup_storage();

        let messages = storage.get(1).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(storage.queue.len(), 2);
        assert_eq!(storage.in_flight.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_storage_get_more_than_available() {
        let mut storage = setup_storage();

        let messages = storage.get(5).await.unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(storage.queue.len(), 0);
        assert_eq!(storage.in_flight.len(), 3);
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete() {
        let mut storage = setup_storage();

        let messages = storage.get(2).await.unwrap();
        storage.delete(vec![messages[0].id.to_string(), messages[1].id.to_string()]).await.unwrap();
        assert_eq!(storage.in_flight.len(), 0);
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete_non_existent() {
        let mut storage = setup_storage();

        let _messages = storage.get(2).await.unwrap();
        storage.delete(vec!["non-existent-id".to_string()]).await.unwrap();
        assert_eq!(storage.in_flight.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_storage_delete_duplicate() {
        let mut storage = setup_storage();

        let messages = storage.get(2).await.unwrap();
        storage.delete(vec![messages[0].id.to_string(), messages[0].id.to_string()]).await.unwrap();
        assert_eq!(storage.in_flight.len(), 1);
    }
}
