use axum::body::Body;
use axum::response::Response;
use axum::routing::RouterIntoService;
use axum::Router;
use http::Request;
use std::sync::Arc;
use tlq::api::create_api;
use tlq::services::MessageService;
use tlq::storage::memory::MemoryStorage;
use tower::{Service, ServiceExt};

pub fn setup_test_app() -> Router {
    let store = Arc::new(MemoryStorage::new());
    let service = MessageService::new(store);

    create_api(service)
}

/// Creates a POST request with the specified path and JSON body.
///
/// # Arguments
///
/// * `path` - The URI path for the request (e.g., "/add")
/// * `body_content` - The JSON content to be sent in the request body
///
/// # Returns
///
/// An HTTP POST request with JSON content type and the provided body
#[cfg(test)]
pub fn create_post_request(path: &str, body_content: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri(path)
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&body_content).unwrap()))
        .unwrap()
}

/// Creates a GET request with the specified path.
///
/// # Arguments
///
/// * `path` - The URI path for the request (e.g., "/hello")
///
/// # Returns
///
/// An HTTP GET request with JSON content type and empty body
#[cfg(test)]
pub fn create_get_request(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .method(http::Method::GET)
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::empty())
        .unwrap()
}

/// Sends an HTTP request to the application router and returns the response.
///
/// # Arguments
///
/// * `app` - Mutable reference to the router service
/// * `post_request` - The HTTP request to be sent
///
/// # Returns
///
/// The HTTP response from the application
///
/// # Errors
///
/// This function will panic if the service is not ready or if the request fails
pub async fn send_request(
    mut app: &mut RouterIntoService<Body>,
    post_request: Request<Body>,
) -> Response {
    ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(post_request)
        .await
        .unwrap()
}
