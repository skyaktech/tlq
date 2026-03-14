use crate::common::{create_get_request, create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_stats_empty_queue() {
    let app = setup_test_app();

    let response = app.oneshot(create_get_request("/stats")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    assert_eq!(body_json["ready"], json!(0));
    assert_eq!(body_json["processing"], json!(0));
}

#[tokio::test]
async fn test_stats_with_ready_and_processing_messages() {
    let mut app = setup_test_app().into_service();

    for i in 1..=5 {
        let request = create_post_request("/add", json!({"body": format!("message {}", i)}));
        send_request(&mut app, request).await;
    }

    let request = create_post_request("/get", json!({"count": 2}));
    send_request(&mut app, request).await;

    let response = send_request(&mut app, create_get_request("/stats")).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    assert_eq!(body_json["ready"], json!(3));
    assert_eq!(body_json["processing"], json!(2));
}
