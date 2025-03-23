use crate::api::models::AddMessageRequest;
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
    dbg!(&request);

    match service.add(Message::new(request.body)).await {
        Ok(_) => success("Success".to_string()),
        Err(_) => error(ApiError::BadRequest(None)),
    }
}
