use crate::common::{create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tlq::types::Message;
use tracing::debug;

#[tokio::test]
async fn test_get_messages() {
    let mut app = setup_test_app().into_service();

    for i in 1..=5 {
        let post_request = create_post_request("/add", json!({"body": format!("message {}", i)}));
        send_request(&mut app, post_request).await;
    }

    let get_request = create_post_request("/get", json!({"count": 3}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<Vec<Message>>(&body).unwrap();
    assert_eq!(body_json.len(), 3);

    let regex = regex::Regex::new(r"message \d+").unwrap();
    for message in &body_json {
        debug!("Message body: {}", &message.body);
        assert!(regex.is_match(&message.body));
    }
    let get_request = create_post_request("/get", json!({"count": 10}));
    let response = send_request(&mut app, get_request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<Vec<Message>>(&body).unwrap();
    assert_eq!(body_json.len(), 2);
}
