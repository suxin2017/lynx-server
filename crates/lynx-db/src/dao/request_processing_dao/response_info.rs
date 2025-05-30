use super::common::{HeaderUtils, HttpMessage};
use anyhow::Result;
use axum::response::Response;
use http::StatusCode;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Response information for processing and logging
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseInfo {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
}

impl ResponseInfo {
    /// Create ResponseInfo from axum Response
    pub async fn from_axum_response(response: Response) -> Result<Self> {
        let status_code = response.status().as_u16();
        let headers = HeaderUtils::extract_headers(response.headers());
        let content_type = HeaderUtils::extract_content_type(response.headers());

        let (_parts, body) = response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        let body_vec = body_bytes.to_vec();
        let content_length = Some(body_vec.len() as u64);

        Ok(ResponseInfo {
            status_code,
            headers,
            body: body_vec,
            content_length,
            content_type,
        })
    }

    /// Create ResponseInfo from raw data
    pub fn new(status_code: u16, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        let content_length = Some(body.len() as u64);
        let content_type = headers.get("content-type").cloned();

        ResponseInfo {
            status_code,
            headers,
            body,
            content_length,
            content_type,
        }
    }

    /// Create an error response
    pub fn error_response(status_code: u16, message: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let body = message.as_bytes().to_vec();
        let content_length = Some(body.len() as u64);

        ResponseInfo {
            status_code,
            headers,
            body,
            content_length,
            content_type: Some("text/plain".to_string()),
        }
    }

    /// Create a JSON response
    pub fn json_response<T: Serialize>(status_code: u16, data: &T) -> Result<Self> {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let body = serde_json::to_vec(data)?;
        let content_length = Some(body.len() as u64);

        Ok(ResponseInfo {
            status_code,
            headers,
            body,
            content_length,
            content_type: Some("application/json".to_string()),
        })
    }

    /// Create a redirect response
    pub fn redirect_response(location: &str, status_code: Option<u16>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("location".to_string(), location.to_string());

        let status = status_code.unwrap_or(302);
        let body = vec![];

        ResponseInfo {
            status_code: status,
            headers,
            body,
            content_length: Some(0),
            content_type: None,
        }
    }

    /// Convert to axum Response
    pub fn to_axum_response(self) -> Result<Response> {
        let mut response_builder =
            Response::builder().status(StatusCode::from_u16(self.status_code)?);

        // Add headers
        for (name, value) in self.headers {
            response_builder = response_builder.header(name, value);
        }

        let response = response_builder.body(axum::body::Body::from(self.body))?;
        Ok(response)
    }

    /// Get response body as string (delegates to HttpMessage trait)
    pub fn body_as_string(&self) -> Result<String> {
        HttpMessage::body_as_string(self)
    }

    /// Get response body as JSON (delegates to HttpMessage trait)
    pub fn body_as_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        HttpMessage::body_as_json(self)
    }

    /// Check if response is successful (2xx status codes)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Check if response is a redirect (3xx status codes)
    pub fn is_redirect(&self) -> bool {
        self.status_code >= 300 && self.status_code < 400
    }

    /// Check if response is a client error (4xx status codes)
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// Check if response is a server error (5xx status codes)
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500 && self.status_code < 600
    }

    /// Get the size of the response body in bytes (delegates to HttpMessage trait)
    pub fn body_size(&self) -> usize {
        HttpMessage::body_size(self)
    }

    /// Check if response body is empty (delegates to HttpMessage trait)
    pub fn is_body_empty(&self) -> bool {
        HttpMessage::is_body_empty(self)
    }

    /// Get a specific header value (delegates to HttpMessage trait)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        HttpMessage::get_header(self, name)
    }

    /// Check if response content is JSON (delegates to HttpMessage trait)
    pub fn is_json(&self) -> bool {
        HttpMessage::is_json(self)
    }

    /// Check if response content is HTML (delegates to HttpMessage trait)
    pub fn is_html(&self) -> bool {
        HttpMessage::is_html(self)
    }

    /// Check if response content is plain text (delegates to HttpMessage trait)
    pub fn is_text(&self) -> bool {
        HttpMessage::is_text(self)
    }
}

impl HttpMessage for ResponseInfo {
    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    fn body_data(&self) -> &[u8] {
        &self.body
    }
}

impl Default for ResponseInfo {
    fn default() -> Self {
        ResponseInfo {
            status_code: 200,
            headers: HashMap::new(),
            body: vec![],
            content_length: Some(0),
            content_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_response_info() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let body = b"Hello, world!".to_vec();
        let response = ResponseInfo::new(200, headers, body);

        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_length, Some(13));
        assert_eq!(response.content_type, Some("application/json".to_string()));
        assert!(response.is_success());
    }

    #[test]
    fn test_error_response() {
        let response = ResponseInfo::error_response(404, "Not Found");

        assert_eq!(response.status_code, 404);
        assert_eq!(response.body_as_string().unwrap(), "Not Found");
        assert_eq!(response.content_type, Some("text/plain".to_string()));
        assert!(response.is_client_error());
    }

    #[test]
    fn test_json_response() {
        let data = json!({"message": "success", "code": 200});
        let response = ResponseInfo::json_response(200, &data).unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_type, Some("application/json".to_string()));
        assert!(response.is_success());

        let parsed: serde_json::Value = response.body_as_json().unwrap();
        assert_eq!(parsed["message"], "success");
        assert_eq!(parsed["code"], 200);
    }

    #[test]
    fn test_redirect_response() {
        let response = ResponseInfo::redirect_response("https://example.com", Some(301));

        assert_eq!(response.status_code, 301);
        assert_eq!(
            response.headers.get("location"),
            Some(&"https://example.com".to_string())
        );
        assert!(response.is_redirect());
        assert_eq!(response.body_size(), 0);
    }

    #[test]
    fn test_status_code_checks() {
        let success_response = ResponseInfo::new(200, HashMap::new(), vec![]);
        assert!(success_response.is_success());
        assert!(!success_response.is_redirect());
        assert!(!success_response.is_client_error());
        assert!(!success_response.is_server_error());

        let redirect_response = ResponseInfo::new(302, HashMap::new(), vec![]);
        assert!(!redirect_response.is_success());
        assert!(redirect_response.is_redirect());
        assert!(!redirect_response.is_client_error());
        assert!(!redirect_response.is_server_error());

        let client_error_response = ResponseInfo::new(404, HashMap::new(), vec![]);
        assert!(!client_error_response.is_success());
        assert!(!client_error_response.is_redirect());
        assert!(client_error_response.is_client_error());
        assert!(!client_error_response.is_server_error());

        let server_error_response = ResponseInfo::new(500, HashMap::new(), vec![]);
        assert!(!server_error_response.is_success());
        assert!(!server_error_response.is_redirect());
        assert!(!server_error_response.is_client_error());
        assert!(server_error_response.is_server_error());
    }

    #[tokio::test]
    async fn test_from_axum_response() {
        use axum::body::Body;
        use axum::response::Response;
        use http::StatusCode;

        // Test basic response
        let body = "Hello, World!";
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/plain")
            .header("x-custom-header", "custom-value")
            .body(Body::from(body))
            .unwrap();

        let response_info = ResponseInfo::from_axum_response(response).await.unwrap();

        assert_eq!(response_info.status_code, 200);
        assert_eq!(response_info.body_as_string().unwrap(), "Hello, World!");
        assert_eq!(response_info.content_type, Some("text/plain".to_string()));
        assert_eq!(response_info.content_length, Some(13));
        assert_eq!(
            response_info.get_header("x-custom-header"),
            Some(&"custom-value".to_string())
        );
        assert!(response_info.is_success());
    }

    #[tokio::test]
    async fn test_from_axum_response_json() {
        use axum::body::Body;
        use axum::response::Response;
        use http::StatusCode;
        use serde_json::json;

        let json_data = json!({"message": "success", "code": 200});
        let json_body = serde_json::to_string(&json_data).unwrap();

        let response = Response::builder()
            .status(StatusCode::CREATED)
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let response_info = ResponseInfo::from_axum_response(response).await.unwrap();

        assert_eq!(response_info.status_code, 201);
        assert_eq!(
            response_info.content_type,
            Some("application/json".to_string())
        );
        assert!(response_info.is_success());
        assert!(response_info.is_json());

        let parsed: serde_json::Value = response_info.body_as_json().unwrap();
        assert_eq!(parsed["message"], "success");
        assert_eq!(parsed["code"], 200);
    }

    #[tokio::test]
    async fn test_from_axum_response_empty_body() {
        use axum::body::Body;
        use axum::response::Response;
        use http::StatusCode;

        let response = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("cache-control", "no-cache")
            .body(Body::empty())
            .unwrap();

        let response_info = ResponseInfo::from_axum_response(response).await.unwrap();

        assert_eq!(response_info.status_code, 204);
        assert_eq!(response_info.content_length, Some(0));
        assert!(response_info.is_body_empty());
        assert!(response_info.is_success());
        assert_eq!(
            response_info.get_header("cache-control"),
            Some(&"no-cache".to_string())
        );
    }

    #[tokio::test]
    async fn test_from_axum_response_error() {
        use axum::body::Body;
        use axum::response::Response;
        use http::StatusCode;

        let error_message = "Internal Server Error";
        let response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("content-type", "text/plain")
            .body(Body::from(error_message))
            .unwrap();

        let response_info = ResponseInfo::from_axum_response(response).await.unwrap();

        assert_eq!(response_info.status_code, 500);
        assert_eq!(
            response_info.body_as_string().unwrap(),
            "Internal Server Error"
        );
        assert!(response_info.is_server_error());
        assert!(!response_info.is_success());
    }
}
