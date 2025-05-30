use super::common::{HeaderUtils, HttpMessage};
use anyhow::Result;
use axum::{body::Bytes, extract::Request};
use http_body_util::BodyExt;
use std::collections::HashMap;

/// Request information for matching
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub url: String,
    pub method: String,
    pub host: String,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
}

impl RequestInfo {
    /// Create RequestInfo from axum Request
    pub async fn from_axum_request(mut req: Request<axum::body::Body>) -> Result<Self> {
        let url = req.uri().to_string();
        let method = req.method().to_string();
        let host = req
            .headers()
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();

        let headers = HeaderUtils::extract_headers(req.headers());

        let body = req.body_mut().collect().await?;
        Ok(RequestInfo {
            url,
            method,
            host,
            headers,
            body: body.to_bytes(),
        })
    }

    /// Convert to axum Request
    pub fn to_axum_request(self) -> Result<Request> {
        use http::{HeaderMap, HeaderName, HeaderValue, Method, Uri};
        use std::str::FromStr;

        // Parse the method
        let method = Method::from_str(&self.method)
            .map_err(|e| anyhow::anyhow!("Invalid HTTP method '{}': {}", self.method, e))?;

        // Parse the URI
        let uri = Uri::from_str(&self.url)
            .map_err(|e| anyhow::anyhow!("Invalid URI '{}': {}", self.url, e))?;

        // Build headers
        let mut header_map = HeaderMap::new();
        for (name, value) in self.headers {
            let header_name = HeaderName::from_str(&name)
                .map_err(|e| anyhow::anyhow!("Invalid header name '{}': {}", name, e))?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(|e| anyhow::anyhow!("Invalid header value '{}': {}", value, e))?;
            header_map.insert(header_name, header_value);
        }

        // Create the request
        let mut request = Request::builder().method(method).uri(uri);

        // Add headers
        *request.headers_mut().unwrap() = header_map;

        // Build with body
        let request = request
            .body(axum::body::Body::from(self.body))
            .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;

        Ok(request)
    }

    /// Get request body as string (delegates to HttpMessage trait)
    pub fn body_as_string(&self) -> Result<String> {
        HttpMessage::body_as_string(self)
    }

    /// Get request body as JSON (delegates to HttpMessage trait)
    pub fn body_as_json<T: for<'de> serde::Deserialize<'de>>(&self) -> Result<T> {
        HttpMessage::body_as_json(self)
    }

    /// Get the size of the request body in bytes (delegates to HttpMessage trait)
    pub fn body_size(&self) -> usize {
        HttpMessage::body_size(self)
    }

    /// Check if request body is empty (delegates to HttpMessage trait)
    pub fn is_body_empty(&self) -> bool {
        HttpMessage::is_body_empty(self)
    }

    /// Get a specific header value (delegates to HttpMessage trait)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        HttpMessage::get_header(self, name)
    }

    /// Check if request content is JSON (delegates to HttpMessage trait)
    pub fn is_json(&self) -> bool {
        HttpMessage::is_json(self)
    }

    /// Check if request content is HTML (delegates to HttpMessage trait)
    pub fn is_html(&self) -> bool {
        HttpMessage::is_html(self)
    }

    /// Check if request content is plain text (delegates to HttpMessage trait)
    pub fn is_text(&self) -> bool {
        HttpMessage::is_text(self)
    }
}

impl HttpMessage for RequestInfo {
    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    fn body_data(&self) -> &[u8] {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::body::Bytes;
    use axum::extract::Request;
    use http::Method;
    use serde_json::json;

    /// Helper function to create a RequestInfo for testing
    fn create_test_request_info(
        url: &str,
        method: &str,
        host: &str,
        headers: HashMap<String, String>,
        body: &[u8],
    ) -> RequestInfo {
        RequestInfo {
            url: url.to_string(),
            method: method.to_string(),
            host: host.to_string(),
            headers,
            body: Bytes::from(body.to_vec()),
        }
    }

    #[tokio::test]
    async fn test_from_axum_request_get() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("https://example.com/api/users?page=1")
            .header("host", "example.com")
            .header("user-agent", "test-agent")
            .header("accept", "application/json")
            .body(Body::empty())
            .unwrap();

        let request_info = RequestInfo::from_axum_request(req).await.unwrap();

        assert_eq!(request_info.url, "https://example.com/api/users?page=1");
        assert_eq!(request_info.method, "GET");
        assert_eq!(request_info.host, "example.com");
        assert_eq!(
            request_info.get_header("user-agent"),
            Some(&"test-agent".to_string())
        );
        assert_eq!(
            request_info.get_header("accept"),
            Some(&"application/json".to_string())
        );
        assert!(request_info.is_body_empty());
        assert_eq!(request_info.body_size(), 0);
    }

    #[tokio::test]
    async fn test_from_axum_request_post_with_json() {
        let json_data = json!({"username": "john", "email": "john@example.com"});
        let json_body = serde_json::to_string(&json_data).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/users")
            .header("host", "api.example.com")
            .header("content-type", "application/json")
            .header("content-length", json_body.len().to_string())
            .body(Body::from(json_body.clone()))
            .unwrap();

        let request_info = RequestInfo::from_axum_request(req).await.unwrap();

        assert_eq!(request_info.url, "/api/users");
        assert_eq!(request_info.method, "POST");
        assert_eq!(request_info.host, "api.example.com");
        assert_eq!(
            request_info.get_header("content-type"),
            Some(&"application/json".to_string())
        );
        assert!(!request_info.is_body_empty());
        assert!(request_info.is_json());
        assert_eq!(request_info.body_as_string().unwrap(), json_body);

        let parsed: serde_json::Value = request_info.body_as_json().unwrap();
        assert_eq!(parsed["username"], "john");
        assert_eq!(parsed["email"], "john@example.com");
    }

    #[tokio::test]
    async fn test_from_axum_request_without_host() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let request_info = RequestInfo::from_axum_request(req).await.unwrap();

        assert_eq!(request_info.url, "/test");
        assert_eq!(request_info.method, "GET");
        assert_eq!(request_info.host, ""); // Should be empty when no host header
        assert!(request_info.is_body_empty());
    }

    #[tokio::test]
    async fn test_from_axum_request_with_form_data() {
        let form_data = "username=john&password=secret123";

        let req = Request::builder()
            .method(Method::POST)
            .uri("/login")
            .header("host", "auth.example.com")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let request_info = RequestInfo::from_axum_request(req).await.unwrap();

        assert_eq!(request_info.url, "/login");
        assert_eq!(request_info.method, "POST");
        assert_eq!(request_info.host, "auth.example.com");
        assert_eq!(
            request_info.get_header("content-type"),
            Some(&"application/x-www-form-urlencoded".to_string())
        );
        assert!(!request_info.is_body_empty());
        assert_eq!(request_info.body_as_string().unwrap(), form_data);
        assert_eq!(request_info.body_size(), form_data.len());
    }

    #[test]
    fn test_body_as_string() {
        let text = "Hello, World!";
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let request_info =
            create_test_request_info("/test", "POST", "example.com", headers, text.as_bytes());

        assert_eq!(request_info.body_as_string().unwrap(), text);
        assert!(request_info.is_text());
        assert!(!request_info.is_json());
        assert!(!request_info.is_html());
    }

    #[test]
    fn test_body_as_json() {
        let json_data = json!({"message": "hello", "count": 42});
        let json_string = serde_json::to_string(&json_data).unwrap();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let request_info = create_test_request_info(
            "/api/test",
            "POST",
            "api.example.com",
            headers,
            json_string.as_bytes(),
        );

        assert!(request_info.is_json());
        assert!(!request_info.is_text());
        assert!(!request_info.is_html());

        let parsed: serde_json::Value = request_info.body_as_json().unwrap();
        assert_eq!(parsed["message"], "hello");
        assert_eq!(parsed["count"], 42);
    }

    #[test]
    fn test_body_as_json_invalid() {
        let invalid_json = "{ invalid json }";
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let request_info = create_test_request_info(
            "/api/test",
            "POST",
            "api.example.com",
            headers,
            invalid_json.as_bytes(),
        );

        assert!(request_info.is_json());
        let result: Result<serde_json::Value> = request_info.body_as_json();
        assert!(result.is_err());
    }

    #[test]
    fn test_html_content() {
        let html_content = "<html><body><h1>Hello</h1></body></html>";
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());

        let request_info = create_test_request_info(
            "/page",
            "GET",
            "example.com",
            headers,
            html_content.as_bytes(),
        );

        assert!(request_info.is_html());
        assert!(!request_info.is_json());
        assert!(!request_info.is_text());
        assert_eq!(request_info.body_as_string().unwrap(), html_content);
    }

    #[test]
    fn test_empty_body() {
        let request_info =
            create_test_request_info("/empty", "GET", "example.com", HashMap::new(), &[]);

        assert!(request_info.is_body_empty());
        assert_eq!(request_info.body_size(), 0);
        assert_eq!(request_info.body_as_string().unwrap(), "");
    }

    #[test]
    fn test_header_access() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token123".to_string());
        headers.insert("x-api-key".to_string(), "secret-key".to_string());
        headers.insert("accept".to_string(), "application/json".to_string());

        let request_info =
            create_test_request_info("/api/secure", "GET", "secure.example.com", headers, &[]);

        assert_eq!(
            request_info.get_header("authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            request_info.get_header("x-api-key"),
            Some(&"secret-key".to_string())
        );
        assert_eq!(
            request_info.get_header("accept"),
            Some(&"application/json".to_string())
        );
        assert_eq!(request_info.get_header("non-existent"), None);
    }

    #[test]
    fn test_request_methods() {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

        for method in &methods {
            let request_info =
                create_test_request_info("/test", method, "example.com", HashMap::new(), &[]);

            assert_eq!(request_info.method, *method);
            assert_eq!(request_info.url, "/test");
            assert_eq!(request_info.host, "example.com");
        }
    }

    #[test]
    fn test_url_variations() {
        let urls = [
            "/simple",
            "/path/with/segments",
            "/api/v1/users/123",
            "/search?q=rust&limit=10",
            "https://example.com/external",
            "http://localhost:8080/local",
        ];

        for url in &urls {
            let request_info =
                create_test_request_info(url, "GET", "example.com", HashMap::new(), &[]);

            assert_eq!(request_info.url, *url);
        }
    }

    #[test]
    fn test_binary_body() {
        let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "image/png".to_string());

        let request_info = create_test_request_info(
            "/upload",
            "POST",
            "files.example.com",
            headers,
            &binary_data,
        );

        assert!(!request_info.is_body_empty());
        assert_eq!(request_info.body_size(), binary_data.len());
        assert!(!request_info.is_json());
        assert!(!request_info.is_html());
        assert!(!request_info.is_text());
        assert_eq!(request_info.body_data(), &binary_data);
    }

    #[tokio::test]
    async fn test_large_request_body() {
        let large_text = "x".repeat(10000); // 10KB of data

        let req = Request::builder()
            .method(Method::POST)
            .uri("/upload")
            .header("host", "example.com")
            .header("content-type", "text/plain")
            .body(Body::from(large_text.clone()))
            .unwrap();

        let request_info = RequestInfo::from_axum_request(req).await.unwrap();

        assert_eq!(request_info.body_size(), 10000);
        assert_eq!(request_info.body_as_string().unwrap(), large_text);
        assert!(!request_info.is_body_empty());
    }

    #[tokio::test]
    async fn test_to_axum_request_get() {
        let mut headers = HashMap::new();
        headers.insert("user-agent".to_string(), "test-agent".to_string());
        headers.insert("accept".to_string(), "application/json".to_string());

        let request_info = create_test_request_info(
            "https://example.com/api/users?page=1",
            "GET",
            "example.com",
            headers,
            &[],
        );

        let axum_request = request_info.to_axum_request().unwrap();

        assert_eq!(axum_request.method(), &Method::GET);
        assert_eq!(
            axum_request.uri().to_string(),
            "https://example.com/api/users?page=1"
        );
        assert_eq!(
            axum_request.headers().get("user-agent").unwrap(),
            "test-agent"
        );
        assert_eq!(
            axum_request.headers().get("accept").unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_to_axum_request_post_with_json() {
        let json_data = json!({"username": "john", "email": "john@example.com"});
        let json_body = serde_json::to_string(&json_data).unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("content-length".to_string(), json_body.len().to_string());

        let request_info = create_test_request_info(
            "/api/users",
            "POST",
            "api.example.com",
            headers,
            json_body.as_bytes(),
        );

        let axum_request = request_info.to_axum_request().unwrap();

        assert_eq!(axum_request.method(), &Method::POST);
        assert_eq!(axum_request.uri().to_string(), "/api/users");
        assert_eq!(
            axum_request.headers().get("content-type").unwrap(),
            "application/json"
        );

        // Extract body and verify
        let (_, body) = axum_request.into_parts();
        let body_bytes = body.collect().await.unwrap().to_bytes();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(body_string, json_body);
    }

    #[tokio::test]
    async fn test_to_axum_request_roundtrip() {
        // Create original request
        let json_data = json!({"test": "data", "number": 123});
        let json_body = serde_json::to_string(&json_data).unwrap();

        let original_req = Request::builder()
            .method(Method::PUT)
            .uri("/api/test/456")
            .header("host", "test.example.com")
            .header("content-type", "application/json")
            .header("authorization", "Bearer token123")
            .header("x-custom-header", "custom-value")
            .body(Body::from(json_body.clone()))
            .unwrap();

        // Convert to RequestInfo
        let request_info = RequestInfo::from_axum_request(original_req).await.unwrap();

        // Convert back to axum Request
        let converted_req = request_info.to_axum_request().unwrap();

        // Verify method, URI, and headers
        assert_eq!(converted_req.method(), &Method::PUT);
        assert_eq!(converted_req.uri().to_string(), "/api/test/456");
        assert_eq!(
            converted_req.headers().get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            converted_req.headers().get("authorization").unwrap(),
            "Bearer token123"
        );
        assert_eq!(
            converted_req.headers().get("x-custom-header").unwrap(),
            "custom-value"
        );

        // Verify body
        let (_, body) = converted_req.into_parts();
        let body_bytes = body.collect().await.unwrap().to_bytes();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(body_string, json_body);
    }

    #[test]
    fn test_to_axum_request_invalid_method() {
        let request_info = RequestInfo {
            url: "/test".to_string(),
            method: "".to_string(), // Empty method should definitely be invalid
            host: "example.com".to_string(),
            headers: HashMap::new(),
            body: Bytes::new(),
        };

        let result = request_info.to_axum_request();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid HTTP method")
        );
    }

    #[test]
    fn test_to_axum_request_invalid_uri() {
        let request_info = RequestInfo {
            url: "invalid uri with spaces".to_string(),
            method: "GET".to_string(),
            host: "example.com".to_string(),
            headers: HashMap::new(),
            body: Bytes::new(),
        };

        let result = request_info.to_axum_request();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid URI"));
    }

    #[test]
    fn test_to_axum_request_invalid_header_name() {
        let mut headers = HashMap::new();
        headers.insert("invalid header name".to_string(), "value".to_string());

        let request_info = RequestInfo {
            url: "/test".to_string(),
            method: "GET".to_string(),
            host: "example.com".to_string(),
            headers,
            body: Bytes::new(),
        };

        let result = request_info.to_axum_request();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid header name")
        );
    }

    #[test]
    fn test_to_axum_request_invalid_header_value() {
        let mut headers = HashMap::new();
        headers.insert("valid-name".to_string(), "invalid\x00value".to_string());

        let request_info = RequestInfo {
            url: "/test".to_string(),
            method: "GET".to_string(),
            host: "example.com".to_string(),
            headers,
            body: Bytes::new(),
        };

        let result = request_info.to_axum_request();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid header value")
        );
    }

    #[test]
    fn test_to_axum_request_all_http_methods() {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

        for method in &methods {
            let request_info =
                create_test_request_info("/test", method, "example.com", HashMap::new(), &[]);

            let axum_request = request_info.to_axum_request().unwrap();
            assert_eq!(axum_request.method().as_str(), *method);
        }
    }

    #[tokio::test]
    async fn test_to_axum_request_with_binary_body() {
        let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "image/png".to_string());

        let request_info = create_test_request_info(
            "/upload",
            "POST",
            "files.example.com",
            headers,
            &binary_data,
        );

        let axum_request = request_info.to_axum_request().unwrap();

        assert_eq!(axum_request.method(), &Method::POST);
        assert_eq!(axum_request.uri().to_string(), "/upload");
        assert_eq!(
            axum_request.headers().get("content-type").unwrap(),
            "image/png"
        );

        // Verify binary body
        let (_, body) = axum_request.into_parts();
        let body_bytes = body.collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.to_vec(), binary_data);
    }

    #[test]
    fn test_to_axum_request_empty_body() {
        let request_info =
            create_test_request_info("/empty", "GET", "example.com", HashMap::new(), &[]);

        let axum_request = request_info.to_axum_request().unwrap();
        assert_eq!(axum_request.method(), &Method::GET);
        assert_eq!(axum_request.uri().to_string(), "/empty");
    }
}
