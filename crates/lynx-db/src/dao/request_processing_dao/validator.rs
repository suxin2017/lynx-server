use super::{
    error::{RequestProcessingError, Result},
    handlers::HandlerRule,
    types::{CaptureCondition, CaptureRule, RequestRule, SimpleCaptureCondition},
};
use crate::entities::{capture::CaptureType, handler::HandlerType};
use glob::Pattern;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::collections::HashSet;

/// Validator for request processing rules
pub struct RuleValidator;

impl RuleValidator {
    /// Validate a complete request rule
    pub fn validate_rule(rule: &RequestRule) -> Result<()> {
        // Validate basic rule properties
        Self::validate_rule_name(&rule.name)?;
        Self::validate_priority(rule.priority)?;

        // Validate capture rule
        Self::validate_capture_rule(&rule.capture)?;

        // Validate handlers
        Self::validate_handlers(&rule.handlers)?;

        Ok(())
    }

    /// Validate rule name
    pub fn validate_rule_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Rule name cannot be empty".to_string(),
            });
        }

        if name.len() > 255 {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Rule name cannot exceed 255 characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate rule priority
    pub fn validate_priority(priority: i32) -> Result<()> {
        if priority < 0 {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Rule priority cannot be negative".to_string(),
            });
        }

        if priority > 10000 {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Rule priority cannot exceed 10000".to_string(),
            });
        }

        Ok(())
    }

    /// Validate capture rule
    pub fn validate_capture_rule(capture: &CaptureRule) -> Result<()> {
        Self::validate_capture_condition(&capture.condition)
    }

    /// Validate capture condition recursively
    pub fn validate_capture_condition(condition: &CaptureCondition) -> Result<()> {
        match condition {
            CaptureCondition::Simple(simple) => Self::validate_simple_condition(simple),
            CaptureCondition::Complex(complex) => {
                if complex.conditions.is_empty() {
                    return Err(RequestProcessingError::RuleValidation {
                        reason: "Complex capture rule must have at least one condition".to_string(),
                    });
                }

                // Validate all sub-conditions
                for sub_condition in &complex.conditions {
                    Self::validate_capture_condition(sub_condition)?;
                }

                Ok(())
            }
        }
    }

    /// Validate simple capture condition
    pub fn validate_simple_condition(condition: &SimpleCaptureCondition) -> Result<()> {
        // Validate pattern based on capture type
        match condition.capture_type {
            CaptureType::Glob => {
                Pattern::new(&condition.pattern).map_err(|e| {
                    RequestProcessingError::InvalidCapturePattern {
                        pattern: condition.pattern.clone(),
                        reason: format!("Invalid glob pattern: {}", e),
                    }
                })?;
            }
            CaptureType::Regex => {
                Regex::new(&condition.pattern).map_err(|e| {
                    RequestProcessingError::InvalidCapturePattern {
                        pattern: condition.pattern.clone(),
                        reason: format!("Invalid regex pattern: {}", e),
                    }
                })?;
            }
            CaptureType::Exact | CaptureType::Contains => {
                // No validation needed for exact or contains patterns
            }
        }

        // Validate pattern is not empty
        if condition.pattern.trim().is_empty() {
            return Err(RequestProcessingError::InvalidCapturePattern {
                pattern: condition.pattern.clone(),
                reason: "Pattern cannot be empty".to_string(),
            });
        }

        // Validate method if specified
        if let Some(ref method) = condition.method {
            if !method.trim().is_empty() {
                Self::validate_http_method(method)?;
            }
        }

        // Validate host if specified
        if let Some(ref host) = condition.host {
            if !host.trim().is_empty() {
                Self::validate_host(host)?;
            }
        }

        Ok(())
    }

    /// Validate HTTP method
    pub fn validate_http_method(method: &str) -> Result<()> {
        let valid_methods = [
            "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "TRACE",
        ];

        if !valid_methods.contains(&method.to_uppercase().as_str()) {
            return Err(RequestProcessingError::RuleValidation {
                reason: format!("Invalid HTTP method: {}", method),
            });
        }

        Ok(())
    }

    /// Validate host
    pub fn validate_host(host: &str) -> Result<()> {
        // Basic host validation - allow wildcards and subdomains
        if host.contains("..") {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Host cannot contain consecutive dots".to_string(),
            });
        }

        if host.starts_with('.') || host.ends_with('.') {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Host cannot start or end with a dot".to_string(),
            });
        }

        Ok(())
    }

    /// Validate handlers
    pub fn validate_handlers(handlers: &[HandlerRule]) -> Result<()> {
        if handlers.is_empty() {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Rule must have at least one handler".to_string(),
            });
        }

        // Check for duplicate execution orders
        let mut orders = HashSet::new();
        for handler in handlers {
            if !orders.insert(handler.execution_order) {
                return Err(RequestProcessingError::RuleValidation {
                    reason: format!("Duplicate execution order: {}", handler.execution_order),
                });
            }

            Self::validate_handler(handler)?;
        }

        Ok(())
    }

    /// Validate individual handler
    pub fn validate_handler(handler: &HandlerRule) -> Result<()> {
        // Validate handler name
        if handler.name.trim().is_empty() {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Handler name cannot be empty".to_string(),
            });
        }

        // Validate execution order
        if handler.execution_order < 0 {
            return Err(RequestProcessingError::RuleValidation {
                reason: "Handler execution order cannot be negative".to_string(),
            });
        }

        // Validate configuration based on handler type
        Self::validate_handler_config(handler.handler_type.clone(), &handler.config)?;

        Ok(())
    }

    /// Validate handler configuration
    pub fn validate_handler_config(handler_type: HandlerType, config: &JsonValue) -> Result<()> {
        match handler_type {
            HandlerType::Block => {
                if let Some(status_code) = config.get("statusCode") {
                    if let Some(code) = status_code.as_u64() {
                        if !(100..=599).contains(&code) {
                            return Err(RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "Block".to_string(),
                                reason: format!("Invalid HTTP status code: {}", code),
                            });
                        }
                    }
                }
            }
            HandlerType::ModifyRequest => {
                // Validate modify request configuration
                if let Some(method) = config.get("modifyMethod") {
                    if let Some(method_str) = method.as_str() {
                        Self::validate_http_method(method_str).map_err(|_| {
                            RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "ModifyRequest".to_string(),
                                reason: format!("Invalid HTTP method: {}", method_str),
                            }
                        })?;
                    }
                }

                if let Some(url) = config.get("modifyUrl") {
                    if let Some(url_str) = url.as_str() {
                        if url_str.trim().is_empty() {
                            return Err(RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "ModifyRequest".to_string(),
                                reason: "Modified URL cannot be empty".to_string(),
                            });
                        }
                    }
                }
            }
            HandlerType::ModifyResponse => {
                // Validate modify response configuration
                if let Some(content_type) = config.get("contentType") {
                    if let Some(ct_str) = content_type.as_str() {
                        if ct_str.trim().is_empty() {
                            return Err(RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "ModifyResponse".to_string(),
                                reason: "Content type cannot be empty".to_string(),
                            });
                        }
                    }
                }

                if let Some(body) = config.get("body") {
                    if !body.is_string() && !body.is_object() && !body.is_array() {
                        return Err(RequestProcessingError::InvalidHandlerConfig {
                            handler_type: "ModifyResponse".to_string(),
                            reason: "Response body must be a string, object, or array".to_string(),
                        });
                    }
                }
            }
            HandlerType::LocalFile => {
                if let Some(file_path) = config.get("filePath") {
                    if let Some(path_str) = file_path.as_str() {
                        if path_str.trim().is_empty() {
                            return Err(RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "LocalFile".to_string(),
                                reason: "File path cannot be empty".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(RequestProcessingError::InvalidHandlerConfig {
                        handler_type: "LocalFile".to_string(),
                        reason: "File path is required".to_string(),
                    });
                }
            }
            HandlerType::ProxyForward => {
                // Validate proxy configuration
                if let Some(target_host) = config.get("targetHost") {
                    if let Some(host_str) = target_host.as_str() {
                        if host_str.trim().is_empty() {
                            return Err(RequestProcessingError::InvalidHandlerConfig {
                                handler_type: "redirect".to_string(),
                                reason: "Target host cannot be empty".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_rule_name() {
        assert!(RuleValidator::validate_rule_name("Valid Rule").is_ok());
        assert!(RuleValidator::validate_rule_name("").is_err());
        assert!(RuleValidator::validate_rule_name("   ").is_err());

        let long_name = "a".repeat(256);
        assert!(RuleValidator::validate_rule_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_priority() {
        assert!(RuleValidator::validate_priority(100).is_ok());
        assert!(RuleValidator::validate_priority(0).is_ok());
        assert!(RuleValidator::validate_priority(-1).is_err());
        assert!(RuleValidator::validate_priority(10001).is_err());
    }

    #[test]
    fn test_validate_http_method() {
        assert!(RuleValidator::validate_http_method("GET").is_ok());
        assert!(RuleValidator::validate_http_method("post").is_ok());
        assert!(RuleValidator::validate_http_method("INVALID").is_err());
    }

    #[test]
    fn test_validate_handler_config() {
        // Block handler
        let block_config = json!({
            "statusCode": 404,
            "reason": "Not found"
        });
        assert!(RuleValidator::validate_handler_config(HandlerType::Block, &block_config).is_ok());

        let invalid_block_config = json!({
            "statusCode": 999
        });
        assert!(
            RuleValidator::validate_handler_config(HandlerType::Block, &invalid_block_config)
                .is_err()
        );

        // Local file handler
        let file_config = json!({
            "filePath": "/path/to/file.txt"
        });
        assert!(
            RuleValidator::validate_handler_config(HandlerType::LocalFile, &file_config).is_ok()
        );

        let invalid_file_config = json!({});
        assert!(
            RuleValidator::validate_handler_config(HandlerType::LocalFile, &invalid_file_config)
                .is_err()
        );
    }
}
