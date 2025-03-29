use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddMessageRequest {
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetMessagesRequest {
    pub count: Option<usize>,
}
