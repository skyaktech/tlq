use crate::config;
use crate::storage::traits::Storage;
use crate::types::Message;
use std::sync::Arc;

#[derive(Clone)]
pub struct MessageService {
    store: Arc<dyn Storage>,
}

impl MessageService {
    pub fn new(store: Arc<dyn Storage>) -> MessageService {
        Self { store }
    }
}

impl MessageService {
    pub async fn add(&self, body: String) -> Result<Message, String> {
        if body.len() > config::config().max_message_size {
            return Err("Message body size is too large".to_string());
        }

        let msg = Message::new(body);
        self.store.add(msg.clone()).await?;
        Ok(msg)
    }

    pub async fn get(&self, count: usize) -> Result<Vec<Message>, String> {
        self.store.get(count).await
    }

    pub async fn delete(&self, ids: Vec<String>) -> Result<(), String> {
        Self::validate_ids(&ids)?;

        self.store.delete(ids).await
    }

    pub async fn purge(&self) -> Result<(), String> {
        self.store.purge().await
    }

    pub async fn retry(&self, ids: Vec<String>) -> Result<(), String> {
        Self::validate_ids(&ids)?;

        self.store.retry(ids).await
    }

    fn validate_ids(ids: &Vec<String>) -> Result<(), String> {
        if ids.is_empty() {
            return Err("No message IDs provided".to_string());
        }

        let mut invalid_ids = Vec::new();
        for id in ids {
            let uuid = uuid::Uuid::parse_str(id);
            if uuid.is_err() {
                invalid_ids.push(id);
            }
        }

        if !invalid_ids.is_empty() {
            return Err(format!("Invalid message IDs: {invalid_ids:?}"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemoryStorage;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_size_within_limit_succeeds() {
        let store = Arc::new(MemoryStorage::new());
        let service = MessageService::new(store);

        let body = "A".repeat(config::config().max_message_size);
        let result = service.add(body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_size_over_limit_fails() {
        let store = Arc::new(MemoryStorage::new());
        let service = MessageService::new(store);

        let body = "A".repeat(config::config().max_message_size + 1);
        let result = service.add(body).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Message body size is too large");
    }

    #[tokio::test]
    async fn test_validate_ids() {
        let ids = vec![Uuid::now_v7().to_string()];
        let result = MessageService::validate_ids(&ids);
        assert!(result.is_ok());

        let ids = vec!["invalid".to_string()];
        let result = MessageService::validate_ids(&ids);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid message IDs: [\"invalid\"]");

        let ids = vec![];
        let result = MessageService::validate_ids(&ids);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No message IDs provided");
    }
}
