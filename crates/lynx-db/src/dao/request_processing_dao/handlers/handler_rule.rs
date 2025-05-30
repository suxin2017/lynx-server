use crate::entities::handler::HandlerType;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use utoipa::ToSchema;

/// Handler rule configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HandlerRule {
    pub id: Option<i32>,
    pub handler_type: HandlerType,
    pub name: String,
    pub description: Option<String>,
    pub execution_order: i32,
    pub config: JsonValue,
    pub enabled: bool,
}

impl Default for HandlerRule {
    fn default() -> Self {
        Self {
            id: None,
            handler_type: HandlerType::Block,
            name: "Default Handler".to_string(),
            description: None,
            execution_order: 0,
            config: json!({}),
            enabled: true,
        }
    }
}

impl HandlerRule {
    /// 获取默认的 Handler 模板
    pub fn default_templates() -> Vec<HandlerRule> {
        vec![
            // Block Handler 模板
            HandlerRule {
                id: None,
                handler_type: HandlerType::Block,
                name: "Block Access".to_string(),
                description: Some("Block all requests with 403 Forbidden".to_string()),
                execution_order: 100,
                config: json!({
                    "statusCode": 403,
                    "reason": "Access blocked by proxy"
                }),
                enabled: false,
            },
        ]
    }
}
