use crate::entities::capture::CaptureType;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use utoipa::ToSchema;

use super::handlers::HandlerRule;

/// 逻辑操作符
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// 简单捕获条件
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleCaptureCondition {
    /// 捕获类型（目前只支持Glob）
    pub capture_type: CaptureType,
    /// 匹配模式（Glob格式）
    pub pattern: String,
    /// HTTP方法过滤
    pub method: Option<String>,
    /// 主机过滤
    pub host: Option<String>,
    /// 额外配置
    pub config: JsonValue,
}

/// 复杂捕获规则（支持嵌套逻辑）
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComplexCaptureRule {
    /// 逻辑操作符
    pub operator: LogicalOperator,
    /// 子条件列表
    #[schema(no_recursion)]
    pub conditions: Vec<CaptureCondition>,
}

/// 捕获条件（简单或复杂）
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CaptureCondition {
    Simple(SimpleCaptureCondition),
    Complex(ComplexCaptureRule),
}

/// 完整的捕获规则
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRule {
    pub id: Option<i32>,
    pub condition: CaptureCondition,
}

/// 请求处理规则
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestRule {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub capture: CaptureRule,
    pub handlers: Vec<HandlerRule>,
}

/// Modify request handler configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModifyRequestConfig {
    pub modify_headers: Option<std::collections::HashMap<String, String>>,
    pub modify_body: Option<String>,
    pub modify_method: Option<String>,
    pub modify_url: Option<String>,
}

/// Local file handler configuration
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalFileConfig {
    pub file_path: String,
    pub content_type: Option<String>,
    pub status_code: Option<u16>,
}

impl Default for RequestRule {
    fn default() -> Self {
        Self {
            id: None,
            name: "New Rule".to_string(),
            description: None,
            enabled: true,
            priority: 0,
            capture: CaptureRule::default(),
            handlers: vec![],
        }
    }
}

impl Default for SimpleCaptureCondition {
    fn default() -> Self {
        Self {
            capture_type: CaptureType::Glob,
            pattern: "*".to_string(),
            method: None,
            host: None,
            config: json!({}),
        }
    }
}

impl Default for ComplexCaptureRule {
    fn default() -> Self {
        Self {
            operator: LogicalOperator::And,
            conditions: vec![],
        }
    }
}

impl Default for CaptureCondition {
    fn default() -> Self {
        Self::Simple(SimpleCaptureCondition::default())
    }
}

impl Default for CaptureRule {
    fn default() -> Self {
        Self {
            id: None,
            condition: CaptureCondition::default(),
        }
    }
}

impl SimpleCaptureCondition {
    /// Create a new simple capture condition with glob pattern
    pub fn new_glob(pattern: &str) -> Self {
        Self {
            capture_type: CaptureType::Glob,
            pattern: pattern.to_string(),
            method: None,
            host: None,
            config: json!({}),
        }
    }

    /// Create a new simple capture condition with specific method
    pub fn with_method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    /// Create a new simple capture condition with specific host
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    /// Add configuration to the condition
    pub fn with_config(mut self, config: JsonValue) -> Self {
        self.config = config;
        self
    }
}

impl ComplexCaptureRule {
    /// Create a new AND rule
    pub fn and(conditions: Vec<CaptureCondition>) -> Self {
        Self {
            operator: LogicalOperator::And,
            conditions,
        }
    }

    /// Create a new OR rule
    pub fn or(conditions: Vec<CaptureCondition>) -> Self {
        Self {
            operator: LogicalOperator::Or,
            conditions,
        }
    }

    /// Create a new NOT rule (should only have one condition)
    pub fn not(condition: CaptureCondition) -> Self {
        Self {
            operator: LogicalOperator::Not,
            conditions: vec![condition],
        }
    }

    /// Add a condition to the complex rule
    pub fn add_condition(mut self, condition: CaptureCondition) -> Self {
        self.conditions.push(condition);
        self
    }
}

impl CaptureCondition {
    /// Create a simple condition from a pattern
    pub fn simple(pattern: &str) -> Self {
        Self::Simple(SimpleCaptureCondition::new_glob(pattern))
    }

    /// Create a complex AND condition
    pub fn and(conditions: Vec<CaptureCondition>) -> Self {
        Self::Complex(ComplexCaptureRule::and(conditions))
    }

    /// Create a complex OR condition
    pub fn or(conditions: Vec<CaptureCondition>) -> Self {
        Self::Complex(ComplexCaptureRule::or(conditions))
    }

    /// Create a complex NOT condition
    pub fn not(condition: CaptureCondition) -> Self {
        Self::Complex(ComplexCaptureRule::not(condition))
    }
}

impl CaptureRule {
    /// Create a new capture rule with simple condition
    pub fn simple(pattern: &str) -> Self {
        Self {
            id: None,
            condition: CaptureCondition::simple(pattern),
        }
    }

    /// Create a new capture rule with complex condition
    pub fn complex(condition: CaptureCondition) -> Self {
        Self {
            id: None,
            condition,
        }
    }

    /// Set the ID
    pub fn with_id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_capture_condition_creation() {
        let condition = SimpleCaptureCondition::new_glob("/api/*")
            .with_method("GET")
            .with_host("example.com");

        assert_eq!(condition.capture_type, CaptureType::Glob);
        assert_eq!(condition.pattern, "/api/*");
        assert_eq!(condition.method, Some("GET".to_string()));
        assert_eq!(condition.host, Some("example.com".to_string()));
    }

    #[test]
    fn test_complex_and_rule() {
        let rule = ComplexCaptureRule::and(vec![
            CaptureCondition::simple("/api/*"),
            CaptureCondition::simple("*.json"),
        ]);

        matches!(rule.operator, LogicalOperator::And);
        assert_eq!(rule.conditions.len(), 2);
    }

    #[test]
    fn test_json_serialization() {
        let rule = CaptureRule::complex(CaptureCondition::and(vec![
            CaptureCondition::simple("/api/*"),
            CaptureCondition::Simple(SimpleCaptureCondition::new_glob("*").with_method("GET")),
        ]));

        let json = serde_json::to_string(&rule).expect("Failed to serialize");
        let deserialized: CaptureRule = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(rule.id, deserialized.id);
    }
}
