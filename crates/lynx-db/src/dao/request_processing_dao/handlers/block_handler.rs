use anyhow::Result;
use axum::{extract::Request, response::Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::handler_trait::{HandleRequestType, HandlerTrait};
use crate::dao::request_processing_dao::ResponseInfo;

/// Block handler configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockHandlerConfig {
    pub status_code: Option<u16>,
    pub reason: Option<String>,
}

impl Default for BlockHandlerConfig {
    fn default() -> Self {
        Self {
            status_code: Some(403),
            reason: Some("Access blocked by proxy".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl HandlerTrait for BlockHandlerConfig {
    /// Handles an incoming HTTP request by blocking it and returning a configured response.
    ///
    /// # Arguments
    /// * `request` - The incoming HTTP request (will be blocked)
    ///
    /// # Returns
    /// Always returns a Response indicating the request is blocked
    async fn handle_request(&self, _request: Request) -> Result<HandleRequestType> {
        let status_code = self.status_code.unwrap_or(403);
        let reason = self.reason.as_deref().unwrap_or("Access blocked by proxy");

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());
        headers.insert("x-blocked-by".to_string(), "lynx-proxy".to_string());

        let body = reason.as_bytes().to_vec();
        let content_length = Some(body.len() as u64);

        let response_info = ResponseInfo {
            status_code,
            headers,
            body,
            content_length,
            content_type: Some("text/plain".to_string()),
        };

        Ok(HandleRequestType::Response(response_info))
    }

    /// Handles an outgoing HTTP response.
    /// Since this is a blocking handler, responses are typically not processed,
    /// but we pass them through unchanged if they somehow reach this point.
    ///
    /// # Arguments
    /// * `response` - The HTTP response to process
    ///
    /// # Returns
    /// Returns the response unchanged
    async fn handle_response(&self, response: Response) -> Result<Option<Response>> {
        Ok(Some(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Method;

    #[tokio::test]
    async fn test_block_handler_default_config() {
        let handler = BlockHandlerConfig::default();
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://example.com/test")
            .body(Body::empty())
            .unwrap();

        let result = handler.handle_request(request).await.unwrap();

        match result {
            HandleRequestType::Response(response_info) => {
                assert_eq!(response_info.status_code, 403);
                assert_eq!(
                    response_info.body_as_string().unwrap(),
                    "Access blocked by proxy"
                );
                assert_eq!(response_info.content_type, Some("text/plain".to_string()));
                assert_eq!(
                    response_info.get_header("x-blocked-by"),
                    Some(&"lynx-proxy".to_string())
                );
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[tokio::test]
    async fn test_block_handler_custom_config() {
        let handler = BlockHandlerConfig {
            status_code: Some(451),
            reason: Some("Blocked for legal reasons".to_string()),
        };

        let request = Request::builder()
            .method(Method::POST)
            .uri("https://example.com/api")
            .body(Body::empty())
            .unwrap();

        let result = handler.handle_request(request).await.unwrap();

        match result {
            HandleRequestType::Response(response_info) => {
                assert_eq!(response_info.status_code, 451);
                assert_eq!(
                    response_info.body_as_string().unwrap(),
                    "Blocked for legal reasons"
                );
                assert_eq!(response_info.content_type, Some("text/plain".to_string()));
                assert_eq!(
                    response_info.get_header("x-blocked-by"),
                    Some(&"lynx-proxy".to_string())
                );
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[tokio::test]
    async fn test_block_handler_response_passthrough() {
        let handler = BlockHandlerConfig::default();
        let response = Response::builder()
            .status(200)
            .body(Body::from("Test response"))
            .unwrap();

        let result = handler.handle_response(response).await.unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_block_handler_serialization() {
        let handler = BlockHandlerConfig {
            status_code: Some(403),
            reason: Some("Custom block message".to_string()),
        };

        let json = serde_json::to_string(&handler).unwrap();
        let deserialized: BlockHandlerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(handler.status_code, deserialized.status_code);
        assert_eq!(handler.reason, deserialized.reason);
    }
}
