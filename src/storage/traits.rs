use crate::types::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn add(&self, msg: Message) -> Result<(), String>;
    async fn get(&self, count: usize) -> Result<Vec<Message>, String>;
    async fn delete(&self, ids: Vec<String>) -> Result<(), String>;
    async fn purge(&self) -> Result<(), String>;
    async fn retry(&self, ids: Vec<String>) -> Result<(), String>;
}
