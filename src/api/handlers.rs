use crate::api::models::{AddMessageRequest, DeleteMessagesRequest, GetMessagesRequest};
use crate::services::MessageService;
use crate::types::Message;
use axum::extract::State;
use axum::Json;
use skyak_axum_core::errors::ApiError;
use skyak_axum_core::https::{error, success, ApiResponse};

pub async fn add_message(
    State(service): State<MessageService>,
    Json(request): Json<AddMessageRequest>,
) -> ApiResponse<String> {
    match service.add(Message::new(request.body)).await {
        Ok(_) => success("Success".to_string()),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}

pub async fn get_messages(
    State(service): State<MessageService>,
    Json(request): Json<GetMessagesRequest>,
) -> ApiResponse<Vec<Message>> {
    let count = request.count.unwrap_or(1);
    match service.get(count).await {
        Ok(messages) => success(messages),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}

pub async fn delete_messages(
    State(service): State<MessageService>,
    Json(request): Json<DeleteMessagesRequest>,
) -> ApiResponse<String> {
    let ids = request.ids;
    match service.delete(ids).await {
        Ok(_) => success("Success".to_string()),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}

pub async fn purge_messages(State(service): State<MessageService>) -> ApiResponse<String> {
    match service.purge().await {
        Ok(_) => success("Success".to_string()),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}

pub async fn retry_messages(
    State(service): State<MessageService>,
    Json(request): Json<DeleteMessagesRequest>,
) -> ApiResponse<String> {
    let ids = request.ids;
    match service.retry(ids).await {
        Ok(_) => success("Success".to_string()),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}
