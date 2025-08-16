use crate::common::{create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tlq::types::Message;

#[tokio::test]
async fn test_retry_messages_and_get_same_message() {
    let mut app = setup_test_app().into_service();

    // Add a message
    let add_request = create_post_request("/add", json!({"body": "message to retry"}));
    send_request(&mut app, add_request).await;

    // Get the message
    let get_request = create_post_request("/get", json!({"count": 1}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let messages: Vec<Message> = serde_json::from_slice(&body).unwrap();
    assert!(!messages.is_empty());

    // Retry the message
    let retry_request = create_post_request("/retry", json!({"ids": [messages[0].id]}));
    let response = send_request(&mut app, retry_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result, json!("Success"));

    // Verify the same message can be retrieved after retry
    let get_request = create_post_request("/get", json!({"count": 1}));
    let response = send_request(&mut app, get_request).await;

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let messages_after_retry: Vec<Message> = serde_json::from_slice(&body).unwrap();
    assert_eq!(messages_after_retry.len(), 1);
    assert_eq!(messages_after_retry[0].id, messages[0].id);
}
