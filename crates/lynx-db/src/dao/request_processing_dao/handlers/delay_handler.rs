use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Delay handler configuration for adding request processing delays
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelayHandlerConfig {
    /// Delay duration in milliseconds
    pub delay_ms: u64,
    /// Optional variance in milliseconds for random delay (±variance)
    pub variance_ms: Option<u64>,
    /// Whether to delay before or after processing the request
    pub delay_type: DelayType,
}

/// Type of delay to apply
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum DelayType {
    /// Delay before processing the request
    #[default]
    BeforeRequest,
    /// Delay after processing the request
    AfterRequest,
    /// Delay both before and after processing
    Both,
}

impl Default for DelayHandlerConfig {
    fn default() -> Self {
        Self {
            delay_ms: 1000, // Default 1 second delay
            variance_ms: None,
            delay_type: DelayType::BeforeRequest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_handler_serialization() {
        let handler = DelayHandlerConfig {
            delay_ms: 2000,
            variance_ms: Some(500),
            delay_type: DelayType::Both,
        };

        let json = serde_json::to_string(&handler).unwrap();
        let deserialized: DelayHandlerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(handler.delay_ms, deserialized.delay_ms);
        assert_eq!(handler.variance_ms, deserialized.variance_ms);
        match (handler.delay_type, deserialized.delay_type) {
            (DelayType::Both, DelayType::Both) => (),
            _ => panic!("DelayType not properly deserialized"),
        }
    }

    #[test]
    fn test_delay_handler_default() {
        let handler = DelayHandlerConfig::default();
        assert_eq!(handler.delay_ms, 1000);
        assert_eq!(handler.variance_ms, None);
        match handler.delay_type {
            DelayType::BeforeRequest => (),
            _ => panic!("Default DelayType should be BeforeRequest"),
        }
    }

    #[test]
    fn test_delay_type_serialization() {
        let delay_types = vec![
            DelayType::BeforeRequest,
            DelayType::AfterRequest,
            DelayType::Both,
        ];

        for delay_type in delay_types {
            let json = serde_json::to_string(&delay_type).unwrap();
            let deserialized: DelayType = serde_json::from_str(&json).unwrap();
            
            match (&delay_type, &deserialized) {
                (DelayType::BeforeRequest, DelayType::BeforeRequest) => (),
                (DelayType::AfterRequest, DelayType::AfterRequest) => (),
                (DelayType::Both, DelayType::Both) => (),
                _ => panic!("DelayType serialization failed"),
            }
        }
    }
}
