use skyak_axum_core::https::{success, ApiResponse};

pub async fn check() -> ApiResponse<String> {
    success("Hello World".to_string())
}
