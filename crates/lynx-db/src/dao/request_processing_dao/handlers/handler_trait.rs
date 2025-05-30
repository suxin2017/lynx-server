use anyhow::Result;
use axum::{extract::Request, response::Response};

use crate::dao::request_processing_dao::{RequestInfo, ResponseInfo};

/// Represents the type of result returned by request handling operations.
/// This enum allows handlers to return either processed request information
/// or response information based on the handling logic.
pub enum HandleRequestType {
    /// Contains processed request information
    Request(RequestInfo),
    /// Contains processed response information
    Response(ResponseInfo),
}

/// A trait for handling HTTP requests and responses in the proxy system.
/// Implementors of this trait can process incoming requests and outgoing responses,
/// applying custom logic such as filtering, modification, logging, or blocking.
///
/// All methods are async to support I/O operations like database access, network calls,
/// or other asynchronous processing during request/response handling.
#[async_trait::async_trait]
pub trait HandlerTrait {
    /// Handles an incoming HTTP request asynchronously.
    ///
    /// This method processes the request and returns either:
    /// - Modified request information that should continue through the proxy
    /// - Response information if the request should be handled directly (e.g., blocked or redirected)
    ///
    /// # Arguments
    /// * `request` - The incoming HTTP request to be processed
    ///
    /// # Returns
    /// * `Ok(HandleRequestType::Request(RequestInfo))` - Continue processing with modified request
    /// * `Ok(HandleRequestType::Response(ResponseInfo))` - Return response directly (bypass proxy)
    /// * `Err(...)` - An error occurred during processing
    async fn handle_request(&self, request: Request) -> Result<HandleRequestType>;

    /// Handles an outgoing HTTP response asynchronously.
    ///
    /// This method processes the response before it's sent back to the client.
    /// It can modify the response content, headers, or decide to block it entirely.
    ///
    /// # Arguments
    /// * `response` - The HTTP response to be processed
    ///
    /// # Returns
    /// * `Ok(Some(modified_response))` - Return the modified response to the client
    /// * `Ok(None)` - Block/suppress the response (no response sent to client)
    /// * `Err(...)` - An error occurred during processing
    async fn handle_response(&self, response: Response) -> Result<Option<Response>>;
}

#[cfg(test)]
mod tests {
    use crate::dao::request_processing_dao::{RequestInfo, ResponseInfo};
    use axum::body::Body;
    use axum::http::{Method, StatusCode};
    use std::collections::HashMap;

    use super::*;

    struct SimpleHandler;

    #[async_trait::async_trait]
    impl HandlerTrait for SimpleHandler {
        async fn handle_request(&self, request: Request) -> Result<HandleRequestType> {
            // Create a simple RequestInfo manually for the test
            let request_info = RequestInfo {
                url: request.uri().to_string(),
                method: request.method().to_string(),
                host: request
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
                headers: std::collections::HashMap::new(),
                body: axum::body::Bytes::new(),
            };

            Ok(HandleRequestType::Request(request_info))
        }

        async fn handle_response(&self, response: Response) -> Result<Option<Response>> {
            Ok(Some(response))
        }
    }

    /// A blocking handler that returns a 403 Forbidden response for all requests
    struct BlockingHandler;

    #[async_trait::async_trait]
    impl HandlerTrait for BlockingHandler {
        async fn handle_request(&self, _request: Request) -> Result<HandleRequestType> {
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), "text/plain".to_string());

            let response_info = ResponseInfo {
                status_code: 403,
                headers,
                body: b"Access Denied".to_vec(),
                content_length: Some(13),
                content_type: Some("text/plain".to_string()),
            };

            Ok(HandleRequestType::Response(response_info))
        }

        async fn handle_response(&self, response: Response) -> Result<Option<Response>> {
            Ok(Some(response))
        }
    }

    /// A handler that modifies request headers
    struct HeaderModifyingHandler;

    #[async_trait::async_trait]
    impl HandlerTrait for HeaderModifyingHandler {
        async fn handle_request(&self, request: Request) -> Result<HandleRequestType> {
            let mut headers = HashMap::new();

            // Copy existing headers
            for (name, value) in request.headers() {
                if let Ok(value_str) = value.to_str() {
                    headers.insert(name.as_str().to_lowercase(), value_str.to_string());
                }
            }

            // Add custom header
            headers.insert("x-processed-by".to_string(), "lynx-proxy".to_string());

            let request_info = RequestInfo {
                url: request.uri().to_string(),
                method: request.method().to_string(),
                host: request
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
                headers,
                body: axum::body::Bytes::new(),
            };

            Ok(HandleRequestType::Request(request_info))
        }

        async fn handle_response(&self, response: Response) -> Result<Option<Response>> {
            Ok(Some(response))
        }
    }

    /// A handler that blocks responses based on content type
    struct ResponseFilterHandler;

    #[async_trait::async_trait]
    impl HandlerTrait for ResponseFilterHandler {
        async fn handle_request(&self, request: Request) -> Result<HandleRequestType> {
            let request_info = RequestInfo {
                url: request.uri().to_string(),
                method: request.method().to_string(),
                host: request
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or_default()
                    .to_string(),
                headers: HashMap::new(),
                body: axum::body::Bytes::new(),
            };

            Ok(HandleRequestType::Request(request_info))
        }

        async fn handle_response(&self, response: Response) -> Result<Option<Response>> {
            // Block responses with image content type
            if let Some(content_type) = response.headers().get("content-type") {
                if let Ok(content_type_str) = content_type.to_str() {
                    if content_type_str.starts_with("image/") {
                        return Ok(None); // Block image responses
                    }
                }
            }
            Ok(Some(response))
        }
    }

    #[tokio::test]
    async fn test_simple_handler_request() {
        let handler = SimpleHandler;
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://example.com/test")
            .header("host", "example.com")
            .body(Body::empty())
            .unwrap();

        let result = handler.handle_request(request).await.unwrap();

        match result {
            HandleRequestType::Request(request_info) => {
                assert_eq!(request_info.url, "https://example.com/test");
                assert_eq!(request_info.method, "GET");
                assert_eq!(request_info.host, "example.com");
            }
            HandleRequestType::Response(_) => {
                panic!("Expected Request variant, got Response");
            }
        }
    }

    #[tokio::test]
    async fn test_simple_handler_response() {
        let handler = SimpleHandler;
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("test response"))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_blocking_handler() {
        let handler = BlockingHandler;
        let request = Request::builder()
            .method(Method::GET)
            .uri("/blocked")
            .body(Body::empty())
            .unwrap();

        let result = handler.handle_request(request).await.unwrap();

        match result {
            HandleRequestType::Response(response_info) => {
                assert_eq!(response_info.status_code, 403);
                assert_eq!(response_info.body, b"Access Denied");
                assert_eq!(
                    response_info.headers.get("content-type"),
                    Some(&"text/plain".to_string())
                );
            }
            HandleRequestType::Request(_) => {
                panic!("Expected Response variant for blocking handler");
            }
        }
    }

    #[tokio::test]
    async fn test_header_modifying_handler() {
        let handler = HeaderModifyingHandler;
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/data")
            .header("content-type", "application/json")
            .header("authorization", "Bearer token123")
            .body(Body::empty())
            .unwrap();

        let result = handler.handle_request(request).await.unwrap();

        match result {
            HandleRequestType::Request(request_info) => {
                assert_eq!(request_info.url, "/api/data");
                assert_eq!(request_info.method, "POST");

                // Check that custom header was added
                assert_eq!(
                    request_info.headers.get("x-processed-by"),
                    Some(&"lynx-proxy".to_string())
                );

                // Check that original headers were preserved (note: they are lowercased)
                assert_eq!(
                    request_info.headers.get("content-type"),
                    Some(&"application/json".to_string())
                );
                assert_eq!(
                    request_info.headers.get("authorization"),
                    Some(&"Bearer token123".to_string())
                );
            }
            HandleRequestType::Response(_) => {
                panic!("Expected Request variant for header modifying handler");
            }
        }
    }

    #[tokio::test]
    async fn test_response_filter_handler_allows_text() {
        let handler = ResponseFilterHandler;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body(Body::from("<html><body>Hello</body></html>"))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_some(), "Text responses should be allowed");
    }

    #[tokio::test]
    async fn test_response_filter_handler_blocks_images() {
        let handler = ResponseFilterHandler;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "image/png")
            .body(Body::from("fake image data"))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_none(), "Image responses should be blocked");
    }

    #[tokio::test]
    async fn test_response_filter_handler_blocks_jpeg() {
        let handler = ResponseFilterHandler;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "image/jpeg")
            .body(Body::from("fake jpeg data"))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_none(), "JPEG responses should be blocked");
    }

    #[tokio::test]
    async fn test_response_filter_allows_json() {
        let handler = ResponseFilterHandler;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"message": "success"}"#))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_some(), "JSON responses should be allowed");
    }
}
