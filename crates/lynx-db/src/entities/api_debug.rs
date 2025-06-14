//! API Debug Entity for saving debug requests

use async_trait::async_trait;
use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;

/// HTTP method enumeration
#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[sea_orm(string_value = "GET")]
    Get,
    #[sea_orm(string_value = "POST")]
    Post,
    #[sea_orm(string_value = "PUT")]
    Put,
    #[sea_orm(string_value = "DELETE")]
    Delete,
    #[sea_orm(string_value = "PATCH")]
    Patch,
    #[sea_orm(string_value = "HEAD")]
    Head,
    #[sea_orm(string_value = "OPTIONS")]
    Options,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

/// Request status enumeration
#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[serde(rename_all = "camelCase")]
pub enum RequestStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "success")]
    Success,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "timeout")]
    Timeout,
}

impl Default for RequestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "api_debug")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Request name/title
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub name: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request URL
    #[sea_orm(column_type = "Text")]
    pub url: String,    /// Request headers as JSON
    #[sea_orm(column_type = "Json", nullable)]
    pub headers: Option<JsonValue>,
    /// Request body content (binary data)
    #[sea_orm(column_type = "Blob", nullable)]
    pub body: Option<Vec<u8>>,
    /// Content type of request body
    #[sea_orm(column_type = "String(StringLen::N(100))", nullable)]
    pub content_type: Option<String>,
    /// Request timeout in seconds
    pub timeout: Option<i32>,
    /// Request status
    pub status: RequestStatus,
    /// Response status code
    pub response_status: Option<i32>,    /// Response headers as JSON
    #[sea_orm(column_type = "Json", nullable)]
    pub response_headers: Option<JsonValue>,
    /// Response body content (binary data)
    #[sea_orm(column_type = "Blob", nullable)]
    pub response_body: Option<Vec<u8>>,
    /// Response time in milliseconds
    pub response_time: Option<i32>,    /// Error message if request failed
    #[sea_orm(column_type = "Text", nullable)]
    pub error_message: Option<String>,
    #[serde(skip)]
    pub created_at: i64,
    #[serde(skip)]
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Called before insert and update
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().timestamp()),
            updated_at: Set(chrono::Utc::now().timestamp()),
            ..ActiveModelTrait::default()
        }
    }

    /// Called before insert
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = chrono::Utc::now();

        if insert {
            // Only set created_at if it's not already set (for new records)
            if self.created_at.is_not_set() {
                self.created_at = Set(now.timestamp());
            }
        }

        // Always update updated_at on both insert and update
        self.updated_at = Set(now.timestamp());

        Ok(self)
    }
}

impl Model {    /// Create a new API debug request
    pub fn new_request(name: String, method: HttpMethod, url: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            name,
            method,
            url,
            headers: None,
            body: None,
            content_type: None,
            timeout: Some(30), // Default 30 seconds
            status: RequestStatus::Pending,
            response_status: None,
            response_headers: None,
            response_body: None,
            response_time: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the request was successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, RequestStatus::Success)
    }

    /// Check if the request failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, RequestStatus::Failed | RequestStatus::Timeout)
    }

    /// Get response status code as string
    pub fn response_status_text(&self) -> String {
        match self.response_status {
            Some(code) => code.to_string(),
            None => "N/A".to_string(),
        }
    }

    /// Get formatted response time
    pub fn formatted_response_time(&self) -> String {
        match self.response_time {
            Some(time) => format!("{}ms", time),
            None => "N/A".to_string(),
        }
    }

    /// Set request body from string
    pub fn set_body_from_string(&mut self, body: String) {
        self.body = Some(body.into_bytes());
    }

    /// Get request body as string (if it's valid UTF-8)
    pub fn get_body_as_string(&self) -> Option<String> {
        self.body.as_ref().and_then(|bytes| String::from_utf8(bytes.clone()).ok())
    }

    /// Set request body from bytes
    pub fn set_body_from_bytes(&mut self, body: Vec<u8>) {
        self.body = Some(body);
    }

    /// Get request body as bytes
    pub fn get_body_as_bytes(&self) -> Option<&[u8]> {
        self.body.as_ref().map(|v| v.as_slice())
    }

    /// Set response body from string
    pub fn set_response_body_from_string(&mut self, body: String) {
        self.response_body = Some(body.into_bytes());
    }

    /// Get response body as string (if it's valid UTF-8)
    pub fn get_response_body_as_string(&self) -> Option<String> {
        self.response_body.as_ref().and_then(|bytes| String::from_utf8(bytes.clone()).ok())
    }

    /// Set response body from bytes
    pub fn set_response_body_from_bytes(&mut self, body: Vec<u8>) {
        self.response_body = Some(body);
    }

    /// Get response body as bytes
    pub fn get_response_body_as_bytes(&self) -> Option<&[u8]> {
        self.response_body.as_ref().map(|v| v.as_slice())
    }

    /// Check if request body is likely text (valid UTF-8)
    pub fn is_body_text(&self) -> bool {
        self.get_body_as_string().is_some()
    }

    /// Check if response body is likely text (valid UTF-8)
    pub fn is_response_body_text(&self) -> bool {
        self.get_response_body_as_string().is_some()
    }

    /// Set request body from JSON value
    pub fn set_body_from_json(&mut self, json: &JsonValue) -> Result<(), serde_json::Error> {
        let json_string = serde_json::to_string(json)?;
        self.body = Some(json_string.into_bytes());
        Ok(())
    }

    /// Get request body as JSON value
    pub fn get_body_as_json(&self) -> Result<Option<JsonValue>, serde_json::Error> {
        match self.get_body_as_string() {
            Some(body_str) => {
                let json_value: JsonValue = serde_json::from_str(&body_str)?;
                Ok(Some(json_value))
            }
            None => Ok(None),
        }
    }

    /// Set response body from JSON value
    pub fn set_response_body_from_json(&mut self, json: &JsonValue) -> Result<(), serde_json::Error> {
        let json_string = serde_json::to_string(json)?;
        self.response_body = Some(json_string.into_bytes());
        Ok(())
    }

    /// Get response body as JSON value
    pub fn get_response_body_as_json(&self) -> Result<Option<JsonValue>, serde_json::Error> {
        match self.get_response_body_as_string() {
            Some(body_str) => {
                let json_value: JsonValue = serde_json::from_str(&body_str)?;
                Ok(Some(json_value))
            }
            None => Ok(None),
        }
    }

    /// Check if request body is valid JSON
    pub fn is_body_json(&self) -> bool {
        self.get_body_as_json().is_ok()
    }

    /// Check if response body is valid JSON
    pub fn is_response_body_json(&self) -> bool {
        self.get_response_body_as_json().is_ok()
    }

    /// Get formatted request body as pretty JSON string
    pub fn get_body_as_pretty_json(&self) -> Result<Option<String>, serde_json::Error> {
        match self.get_body_as_json()? {
            Some(json) => {
                let pretty = serde_json::to_string_pretty(&json)?;
                Ok(Some(pretty))
            }
            None => Ok(None),
        }
    }

    /// Get formatted response body as pretty JSON string
    pub fn get_response_body_as_pretty_json(&self) -> Result<Option<String>, serde_json::Error> {
        match self.get_response_body_as_json()? {
            Some(json) => {
                let pretty = serde_json::to_string_pretty(&json)?;
                Ok(Some(pretty))
            }
            None => Ok(None),
        }
    }
}
