use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Modify request handler configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModifyRequestConfig {
    pub modify_headers: Option<std::collections::HashMap<String, String>>,
    pub modify_body: Option<String>,
    pub modify_method: Option<String>,
    pub modify_url: Option<String>,
}
