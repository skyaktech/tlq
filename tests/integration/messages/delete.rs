use crate::common::{create_post_request, send_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tlq::types::Message;

#[tokio::test]
async fn test_delete_messages() {
    let mut app = setup_test_app().into_service();

    let add_request = create_post_request("/add", json!({"body": "message 1"}));
    send_request(&mut app, add_request).await;

    let get_request = create_post_request("/get", json!({"count": 1}));
    let response = send_request(&mut app, get_request).await;
    let body = serde_json::from_slice::<Vec<Message>>(
        &response.into_body().collect().await.unwrap().to_bytes(),
    )
    .unwrap();

    let delete_request = create_post_request("/delete", json!({"ids": [body[0].id]}));
    let response = send_request(&mut app, delete_request).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_incorrect_id() {
    let mut app = setup_test_app().into_service();

    let delete_request =
        create_post_request("/delete", json!({"ids": ["invalid-id1", "invalid-id2"]}));
    let response = send_request(&mut app, delete_request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(
        body_text,
        "Invalid message IDs: [\"invalid-id1\", \"invalid-id2\"]"
    );
}
