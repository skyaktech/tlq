use skyak_axum_core::https::{success, ApiResponse};

use crate::uptime::uptime_human;

pub async fn check() -> ApiResponse<String> {
    let version = env!("CARGO_PKG_VERSION");
    success(format!(
        "Hello TLQ v{} (uptime: {})",
        version,
        uptime_human()
    ))
}
