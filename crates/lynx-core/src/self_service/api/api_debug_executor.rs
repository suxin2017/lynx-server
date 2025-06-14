use crate::client::reqwest_client::ReqwestClient;
use crate::self_service::{
    RouteState,
    utils::{ResponseDataWrapper, ok},
};
use axum::{Json, extract::State, http::StatusCode};
use lynx_db::dao::api_debug_dao::{ApiDebugDao, CreateApiDebugRequest, UpdateApiDebugRequest};
use lynx_db::entities::api_debug::{HttpMethod, RequestStatus};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// Request for executing an API debug entry
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteApiDebugRequest {
    /// Name of the API request
    pub name: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Target URL
    pub url: String,
    /// HTTP headers as key-value pairs
    pub headers: Option<HashMap<String, String>>,
    /// Request body
    pub body: Option<String>,
    /// Content type header
    pub content_type: Option<String>,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
}

/// Response for executing an API debug entry
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteApiDebugResponse {
    /// ID of the created debug entry
    pub id: i32,
    /// Request execution status
    pub status: RequestStatus,
    /// HTTP response status code
    pub response_status: Option<i32>,
    /// Response headers as JSON
    pub response_headers: Option<JsonValue>,
    /// Response body
    pub response_body: Option<String>,
    /// Response time in milliseconds
    pub response_time: Option<i32>,
    /// Error message if request failed
    pub error_message: Option<String>,
}

#[utoipa::path(
    post,
    path = "/execute",
    tags = ["API Debug Executor"],
    request_body = ExecuteApiDebugRequest,
    responses(
        (status = 200, description = "API request executed successfully", body = ResponseDataWrapper<ExecuteApiDebugResponse>),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Failed to execute API request")
    )
)]
async fn execute_api_request(
    State(RouteState { db, client, .. }): State<RouteState>,
    Json(request): Json<ExecuteApiDebugRequest>,
) -> Result<Json<ResponseDataWrapper<ExecuteApiDebugResponse>>, StatusCode> {
    let dao = ApiDebugDao::new(db.clone());

    // Convert headers to HeaderMap
    let mut header_map = HeaderMap::new();
    if let Some(headers) = &request.headers {
        for (key, value) in headers {
            if let (Ok(header_name), Ok(header_value)) =
                (HeaderName::from_str(key), HeaderValue::from_str(value))
            {
                header_map.insert(header_name, header_value);
            }
        }
    }

    // Set content-type if provided
    if let Some(content_type) = &request.content_type {
        if let Ok(header_value) = HeaderValue::from_str(content_type) {
            header_map.insert("content-type", header_value);
        }
    }

    // Create the initial debug entry
    let create_request = CreateApiDebugRequest {
        name: request.name.clone(),
        method: request.method.clone(),
        url: request.url.clone(),
        headers: request
            .headers
            .as_ref()
            .map(|h| serde_json::to_value(h).unwrap_or(JsonValue::Null)),
        body: request.body.clone(),
        content_type: request.content_type.clone(),
        timeout: request.timeout.map(|t| t as i32),
    };

    let debug_entry = dao.create(create_request).await.map_err(|e| {
        tracing::error!("Failed to create debug entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Start timing
    let start_time = Instant::now();

    // Execute the HTTP request
    let (status, response_status, response_headers, response_body, error_message) =
        execute_http_request(&client, &request, header_map).await;

    // Calculate response time
    let response_time = start_time.elapsed().as_millis() as i32;

    // Update the debug entry with results
    let update_request = UpdateApiDebugRequest {
        name: None,
        method: None,
        url: None,
        headers: None,
        body: None,
        content_type: None,
        timeout: None,
        status: Some(status.clone()),
        response_status,
        response_headers: response_headers.clone(),
        response_body: response_body.clone(),
        response_time: Some(response_time),
        error_message: error_message.clone(),
    };

    let updated_entry = dao
        .update(debug_entry.id, update_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update debug entry: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let final_entry = updated_entry.unwrap_or(debug_entry);

    let response = ExecuteApiDebugResponse {
        id: final_entry.id,
        status,
        response_status,
        response_headers,
        response_body,
        response_time: Some(response_time),
        error_message,
    };

    Ok(Json(ok(response)))
}

/// Execute HTTP request and return results
async fn execute_http_request(
    client: &ReqwestClient,
    request: &ExecuteApiDebugRequest,
    headers: HeaderMap,
) -> (
    RequestStatus,
    Option<i32>,
    Option<JsonValue>,
    Option<String>,
    Option<String>,
) {
    let timeout_duration = std::time::Duration::from_secs(request.timeout.unwrap_or(30));

    let mut req_builder = match request.method {
        HttpMethod::Get => client.client().get(&request.url),
        HttpMethod::Post => client.client().post(&request.url),
        HttpMethod::Put => client.client().put(&request.url),
        HttpMethod::Delete => client.client().delete(&request.url),
        HttpMethod::Patch => client.client().patch(&request.url),
        HttpMethod::Head => client.client().head(&request.url),
        HttpMethod::Options => client
            .client()
            .request(reqwest::Method::OPTIONS, &request.url),
    };

    // Add headers
    req_builder = req_builder.headers(headers);

    // Add body if present
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Set timeout
    req_builder = req_builder.timeout(timeout_duration);

    // Execute request
    match req_builder.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;

            // Convert response headers to JSON
            let response_headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let response_headers_json = serde_json::to_value(response_headers).ok();

            // Get response body
            match response.text().await {
                Ok(body) => {
                    let status = if (200..300).contains(&(status_code as u16)) {
                        RequestStatus::Success
                    } else {
                        RequestStatus::Failed
                    };

                    (
                        status,
                        Some(status_code),
                        response_headers_json,
                        Some(body),
                        None,
                    )
                }
                Err(e) => {
                    tracing::error!("Failed to read response body: {}", e);
                    (
                        RequestStatus::Failed,
                        Some(status_code),
                        response_headers_json,
                        None,
                        Some(e.to_string()),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("HTTP request failed: {}", e);
            (RequestStatus::Failed, None, None, None, Some(e.to_string()))
        }
    }
}

/// Create router for API debug executor
pub fn router(state: RouteState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(execute_api_request))
        .with_state(state)
}
