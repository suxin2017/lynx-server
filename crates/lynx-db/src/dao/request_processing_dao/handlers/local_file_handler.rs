use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Local file handler configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalFileConfig {
    pub file_path: String,
    pub content_type: Option<String>,
    pub status_code: Option<u16>,
}
