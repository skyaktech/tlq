use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

/// Represents the current state of a message in the queue
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MessageState {
    /// Message is available for processing
    Ready,
    /// Message is locked and being processed by a consumer
    Processing,
    /// Message has been processed and can be removed from the queue
    Done,
}

/// Represents a message in the queue system.
/// Uses UUID v7 for time-ordered message IDs with embedded timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier with embedded timestamp (UUID v7)
    pub id: Uuid,
    /// Content of the message
    pub body: String,
    /// Current state of the message
    pub state: MessageState,
    /// Unix timestamp in milliseconds when the message lock expires.
    /// None means the message is not locked
    pub lock_until: Option<i64>,
    /// Number of processing attempts made on this message
    pub retry_count: i32,
}

impl Message {
    /// Creates a new message with the given body.
    /// Initializes with:
    /// - UUID v7 for time-ordered ID
    /// - Ready state
    /// - No lock
    /// - Zero retry count
    ///
    /// # Arguments
    ///
    /// * `body` - The content of the message
    ///
    /// # Examples
    ///
    /// ```
    /// use tlq::types::{Message, MessageState};
    ///
    /// let msg = Message::new("Hello world".to_string());
    /// assert!(matches!(msg.state, MessageState::Ready));
    /// assert_eq!(msg.retry_count, 0);
    /// ```
    pub fn new(body: String) -> Message {
        Message {
            id: Uuid::now_v7(),
            body,
            state: MessageState::Ready,
            lock_until: None,
            retry_count: 0,
        }
    }
}

#[async_trait]
pub trait Storage {
    async fn add(&mut self, msg: Message) -> Result<(), String>;
    async fn get(&mut self, count: usize) -> Result<Vec<Message>, String>;
    async fn delete(&mut self, id: String) -> Result<(), String>;
    async fn purge(&mut self) -> Result<(), String>;
    async fn retry(&mut self, id: String) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let msg = Message::new("Hello world".to_string());
        assert_eq!(msg.body, "Hello world");
        assert!(matches!(msg.state, MessageState::Ready));
        assert_eq!(msg.retry_count, 0);
    }
}
