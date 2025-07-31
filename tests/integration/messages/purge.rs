use crate::common::{create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tlq::types::Message;

#[tokio::test]
async fn test_purge_messages() {
    let mut app = setup_test_app().into_service();

    // Add some messages
    for i in 1..=5 {
        let add_request = create_post_request("/add", json!({"body": format!("message {}", i)}));
        send_request(&mut app, add_request).await;
    }

    // Purge all messages
    let purge_request = create_post_request("/purge", json!({}));
    let response = send_request(&mut app, purge_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Verify that no messages are left
    let get_request = create_post_request("/get", json!({"count": 10}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<Vec<Message>>(&body).unwrap();
    assert!(body_json.is_empty());
}

#[tokio::test]
async fn test_purge_with_processing() {
    let mut app = setup_test_app().into_service();

    // Add some messages
    for i in 1..=5 {
        let add_request = create_post_request("/add", json!({"body": format!("message {}", i)}));
        send_request(&mut app, add_request).await;
    }

    // Get 2 messages
    let get_request = create_post_request("/get", json!({"count": 2}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let messages = serde_json::from_slice::<Vec<Message>>(&body).unwrap();
    assert_eq!(messages.len(), 2);

    // Purge all messages
    let purge_request = create_post_request("/purge", json!({}));
    let response = send_request(&mut app, purge_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Retry 2 fetched messages
    let message_ids: Vec<_> = messages.iter().map(|m| m.id).collect();
    let retry_request = create_post_request("/retry", json!({"ids": message_ids}));
    let response = send_request(&mut app, retry_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Verify that no messages are left
    let get_request = create_post_request("/get", json!({"count": 10}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<Vec<Message>>(&body).unwrap();
    assert!(body_json.is_empty());
}
