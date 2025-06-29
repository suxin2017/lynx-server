use crate::entities::handler::{self, HandlerType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    BlockHandlerConfig, DelayHandlerConfig, DelayType, HtmlScriptInjectorConfig, LocalFileConfig, ModifyRequestConfig,
    modify_response_handler::ModifyResponseConfig, proxy_forward_handler::ProxyForwardConfig,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HandlerRuleType {
    Block(BlockHandlerConfig),
    ModifyRequest(ModifyRequestConfig),
    LocalFile(LocalFileConfig),
    ModifyResponse(ModifyResponseConfig),
    ProxyForward(ProxyForwardConfig),
    HtmlScriptInjector(HtmlScriptInjectorConfig),
    Delay(DelayHandlerConfig),
}

impl From<&handler::Model> for HandlerRuleType {
    fn from(model: &handler::Model) -> Self {
        match model.handler_type {
            HandlerType::Block => {
                let config: BlockHandlerConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::Block(config)
            }
            HandlerType::ModifyRequest => {
                let config: ModifyRequestConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::ModifyRequest(config)
            }
            HandlerType::LocalFile => {
                let config: LocalFileConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::LocalFile(config)
            }
            HandlerType::ModifyResponse => {
                let config: ModifyResponseConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::ModifyResponse(config)
            }
            HandlerType::ProxyForward => {
                let config: ProxyForwardConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::ProxyForward(config)
            }
            HandlerType::HtmlScriptInjector => {
                let config: HtmlScriptInjectorConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::HtmlScriptInjector(config)
            }
            HandlerType::Delay => {
                let config: DelayHandlerConfig =
                    serde_json::from_value(model.config.clone()).unwrap_or_default();
                HandlerRuleType::Delay(config)
            }
        }
    }
}

/// Handler rule configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HandlerRule {
    pub id: Option<i32>,
    pub handler_type: HandlerRuleType,
    pub name: String,
    pub description: Option<String>,
    pub execution_order: i32,
    pub enabled: bool,
}

impl HandlerRule {
    pub fn block_handler(status_code: Option<u16>, reason: Option<String>) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::Block(BlockHandlerConfig {
                status_code,
                reason,
            }),
            name: "Block Access".to_string(),
            description: Some("Block all requests with 403 Forbidden".to_string()),
            execution_order: 100,
            enabled: true,
        }
    }

    pub fn local_file_handler(
        file_path: String,
        content_type: Option<String>,
        status_code: Option<u16>,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::LocalFile(LocalFileConfig {
                file_path,
                content_type,
                status_code,
            }),
            name: "Local File Handler".to_string(),
            description: Some("Serve local files from filesystem".to_string()),
            execution_order: 50,
            enabled: true,
        }
    }

    pub fn modify_request_handler(
        modify_headers: Option<std::collections::HashMap<String, String>>,
        modify_body: Option<String>,
        modify_method: Option<String>,
        modify_url: Option<String>,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::ModifyRequest(ModifyRequestConfig {
                modify_headers,
                modify_body,
                modify_method,
                modify_url,
            }),
            name: "Modify Request Handler".to_string(),
            description: Some("Modify request headers, body, method, or URL".to_string()),
            execution_order: 20,
            enabled: true,
        }
    }

    pub fn modify_response_handler(
        modify_headers: Option<std::collections::HashMap<String, String>>,
        modify_body: Option<String>,
        modify_method: Option<String>,
        modify_status_code: Option<u16>,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::ModifyResponse(ModifyResponseConfig {
                modify_headers,
                modify_body,
                modify_method,
                modify_status_code,
            }),
            name: "Modify Response Handler".to_string(),
            description: Some("Modify response headers, body, method, or status code".to_string()),
            execution_order: 80,
            enabled: true,
        }
    }

    pub fn proxy_forward_handler(
        target_scheme: Option<String>,
        target_authority: Option<String>,
        target_path: Option<String>,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::ProxyForward(ProxyForwardConfig {
                target_scheme,
                target_authority,
                target_path,
            }),
            name: "Proxy Forward Handler".to_string(),
            description: Some("Forward requests to specified proxy target".to_string()),
            execution_order: 10,
            enabled: true,
        }
    }

    pub fn html_script_injector_handler(
        content: Option<String>,
        injection_position: Option<String>,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::HtmlScriptInjector(HtmlScriptInjectorConfig {
                content,
                injection_position: injection_position.or(Some("body-end".to_string())),
            }),
            name: "HTML Content Injector Handler".to_string(),
            description: Some("Inject HTML content into HTML responses".to_string()),
            execution_order: 85,
            enabled: true,
        }
    }

    pub fn delay_handler(
        delay_ms: u64,
        variance_ms: Option<u64>,
        delay_type: DelayType,
    ) -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::Delay(DelayHandlerConfig {
                delay_ms,
                variance_ms,
                delay_type,
            }),
            name: "Delay Handler".to_string(),
            description: Some("Add processing delay to requests".to_string()),
            execution_order: 5, // Execute early to delay before processing
            enabled: true,
        }
    }
}

impl Default for HandlerRule {
    fn default() -> Self {
        Self {
            id: None,
            handler_type: HandlerRuleType::Block(BlockHandlerConfig {
                status_code: Some(403),
                reason: Some("block".to_string()),
            }),
            name: "Default Handler".to_string(),
            description: None,
            execution_order: 0,
            enabled: true,
        }
    }
}
