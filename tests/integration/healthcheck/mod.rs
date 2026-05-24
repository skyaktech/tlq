use crate::common::{create_get_request, setup_test_app};
use http::StatusCode;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app();

    let response = app.oneshot(create_get_request("/hello")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

    let body_str = body_json.as_str().expect("body should be a JSON string");
    let expected_prefix = format!("Hello TLQ v{}", env!("CARGO_PKG_VERSION"));
    assert!(
        body_str.starts_with(&expected_prefix),
        "unexpected body: {body_str}"
    );
    assert!(
        body_str.contains("(uptime: "),
        "unexpected body: {body_str}"
    );
}
