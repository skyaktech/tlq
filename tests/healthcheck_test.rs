use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

mod utils;

#[tokio::test]
async fn test_health_check() {
    let app = utils::setup_test_app();

    let response = app
        .oneshot(utils::create_get_request("/hello"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    assert_eq!(body_json, json!("Hello World"));
}
