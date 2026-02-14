use crate::common::{create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use std::collections::HashSet;
use tlq::types::Message;

#[tokio::test]
async fn test_processing_messages() {
    let mut app = setup_test_app().into_service();

    for i in 1..=5 {
        let post_request = create_post_request("/add", json!({"body": format!("message {}", i)}));
        send_request(&mut app, post_request).await;
    }

    let get_request = create_post_request("/get", json!({"count": 3}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let get_body = response.into_body().collect().await.unwrap().to_bytes();
    let get_json = serde_json::from_slice::<HashSet<Message>>(&get_body).unwrap();
    assert_eq!(get_json.len(), 3);

    let processing_request = create_post_request("/processing", json!(null));
    let response = send_request(&mut app, processing_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let processing_body = response.into_body().collect().await.unwrap().to_bytes();
    let processing_json = serde_json::from_slice::<HashSet<Message>>(&processing_body).unwrap();
    assert_eq!(get_json, processing_json);
}
