use axum::http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

mod utils;

#[tokio::test]
async fn test_add_valid_message_returns_success_response() {
    let app = utils::setup_test_app();

    let response = app
        .oneshot(utils::create_post_request(
            "/add",
            json!({"body": "Hello World"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    assert_eq!(body_json, json!("Success"));
}

#[tokio::test]
async fn test_add_message_with_body_exceeding_size_limit_returns_bad_request() {
    let app = utils::setup_test_app();

    let large_body = "a".repeat(65537);

    let response = app
        .oneshot(utils::create_post_request(
            "/add",
            json!({"body": large_body}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_text, "Message body size is too large");
}
