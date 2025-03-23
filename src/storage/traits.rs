use async_trait::async_trait;
use crate::types::Message;

#[async_trait]
pub trait Storage {
    async fn add(&self, msg: Message) -> Result<(), String>;
    async fn get(&self, count: usize) -> Result<Vec<Message>, String>;
    async fn delete(&self, ids: Vec<String>) -> Result<(), String>;
    async fn purge(&self) -> Result<(), String>;
    async fn retry(&self, ids: Vec<String>) -> Result<(), String>;
}