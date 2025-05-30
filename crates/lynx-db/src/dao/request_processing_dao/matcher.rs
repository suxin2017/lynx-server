use super::{
    error::RequestProcessingError,
    request_info::RequestInfo,
    types::{CaptureCondition, CaptureRule, LogicalOperator, RequestRule, SimpleCaptureCondition},
};
use crate::entities::capture::CaptureType;
use anyhow::Result;
use glob::Pattern;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Compiled pattern for efficient matching
#[derive(Debug, Clone)]
pub enum CompiledPattern {
    Glob(Pattern),
    Regex(Regex),
    Exact(String),
    Contains(String),
}

impl CompiledPattern {
    pub fn matches(&self, text: &str) -> bool {
        match self {
            CompiledPattern::Glob(pattern) => pattern.matches(text),
            CompiledPattern::Regex(regex) => regex.is_match(text),
            CompiledPattern::Exact(exact) => exact == text,
            CompiledPattern::Contains(contains) => text.contains(contains),
        }
    }
}

/// Compiled capture condition for efficient matching
#[derive(Debug, Clone)]
pub struct CompiledCaptureCondition {
    pub pattern: CompiledPattern,
    pub method: Option<String>,
    pub host: Option<String>,
}

/// Compiled capture rule
#[derive(Debug, Clone)]
pub enum CompiledCaptureRule {
    Simple(CompiledCaptureCondition),
    Complex {
        operator: LogicalOperator,
        conditions: Vec<CompiledCaptureRule>,
    },
}

/// Compiled request rule for efficient matching
#[derive(Debug, Clone)]
pub struct CompiledRequestRule {
    pub id: Option<i32>,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub capture: CompiledCaptureRule,
    pub original_rule: RequestRule,
}

/// Cache for compiled rules
type RuleCache = Arc<RwLock<HashMap<i32, CompiledRequestRule>>>;

/// Enhanced rule matcher with performance optimizations and caching
pub struct RuleMatcher {
    cache: RuleCache,
}

impl RuleMatcher {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find matching rules for a request with optimized performance
    pub fn find_matching_rules(
        &self,
        rules: &[RequestRule],
        request: &RequestInfo,
    ) -> Result<Vec<RequestRule>> {
        // Compile rules with caching
        let compiled_rules = self.compile_rules(rules)?;

        let mut matching_rules = Vec::new();

        for compiled_rule in compiled_rules {
            if !compiled_rule.enabled {
                continue;
            }

            // Check if the capture rule matches
            if Self::compiled_capture_matches(&compiled_rule.capture, request)? {
                matching_rules.push(compiled_rule.original_rule);
            }
        }

        // Sort by priority (higher priority first)
        matching_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(matching_rules)
    }

    /// Check if a capture rule matches the request
    pub fn capture_matches(&self, capture: &CaptureRule, request: &RequestInfo) -> Result<bool> {
        let compiled_condition = Self::compile_capture_condition(&capture.condition)?;
        Self::compiled_capture_matches(&compiled_condition, request)
    }

    /// Evaluate a capture condition against a request
    pub fn evaluate_condition(
        &self,
        condition: &CaptureCondition,
        request: &RequestInfo,
    ) -> Result<bool> {
        let compiled_condition = Self::compile_capture_condition(condition)?;
        Self::compiled_capture_matches(&compiled_condition, request)
    }

    /// Clear the rule cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Remove a specific rule from cache
    pub fn invalidate_rule(&self, rule_id: i32) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(&rule_id);
    }

    /// Compile and cache rules for efficient matching
    fn compile_rules(&self, rules: &[RequestRule]) -> Result<Vec<CompiledRequestRule>> {
        let mut compiled_rules = Vec::new();
        let mut cache = self.cache.write().unwrap();

        for rule in rules {
            let rule_id = rule.id.unwrap_or(-1);

            // Check if already cached
            if let Some(cached_rule) = cache.get(&rule_id) {
                // Simple check if rule has been modified
                if cached_rule.name == rule.name && cached_rule.priority == rule.priority {
                    compiled_rules.push(cached_rule.clone());
                    continue;
                }
            }

            // Compile the rule
            let compiled_rule = Self::compile_rule(rule)?;

            // Cache the compiled rule
            if rule_id >= 0 {
                cache.insert(rule_id, compiled_rule.clone());
            }

            compiled_rules.push(compiled_rule);
        }

        Ok(compiled_rules)
    }

    /// Compile a single rule
    fn compile_rule(rule: &RequestRule) -> Result<CompiledRequestRule> {
        let compiled_capture = Self::compile_capture_condition(&rule.capture.condition)?;

        Ok(CompiledRequestRule {
            id: rule.id,
            name: rule.name.clone(),
            enabled: rule.enabled,
            priority: rule.priority,
            capture: compiled_capture,
            original_rule: rule.clone(),
        })
    }

    /// Compile a capture condition
    fn compile_capture_condition(condition: &CaptureCondition) -> Result<CompiledCaptureRule> {
        match condition {
            CaptureCondition::Simple(simple) => {
                let compiled_condition = Self::compile_simple_condition(simple)?;
                Ok(CompiledCaptureRule::Simple(compiled_condition))
            }
            CaptureCondition::Complex(complex) => {
                let mut compiled_conditions = Vec::new();
                for sub_condition in &complex.conditions {
                    compiled_conditions.push(Self::compile_capture_condition(sub_condition)?);
                }

                Ok(CompiledCaptureRule::Complex {
                    operator: complex.operator.clone(),
                    conditions: compiled_conditions,
                })
            }
        }
    }

    /// Compile a simple capture condition
    fn compile_simple_condition(
        condition: &SimpleCaptureCondition,
    ) -> Result<CompiledCaptureCondition> {
        let pattern = match condition.capture_type {
            CaptureType::Glob => {
                let pattern = Pattern::new(&condition.pattern)?;
                CompiledPattern::Glob(pattern)
            }
            CaptureType::Regex => {
                let regex = Regex::new(&condition.pattern)?;
                CompiledPattern::Regex(regex)
            }
            CaptureType::Exact => CompiledPattern::Exact(condition.pattern.clone()),
            CaptureType::Contains => CompiledPattern::Contains(condition.pattern.clone()),
        };

        Ok(CompiledCaptureCondition {
            pattern,
            method: condition.method.clone(),
            host: condition.host.clone(),
        })
    }

    /// Check if compiled capture rule matches request
    fn compiled_capture_matches(
        capture: &CompiledCaptureRule,
        request: &RequestInfo,
    ) -> Result<bool> {
        match capture {
            CompiledCaptureRule::Simple(simple) => Self::compiled_simple_matches(simple, request),
            CompiledCaptureRule::Complex {
                operator,
                conditions,
            } => match operator {
                LogicalOperator::And => {
                    for condition in conditions {
                        if !Self::compiled_capture_matches(condition, request)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                LogicalOperator::Or => {
                    for condition in conditions {
                        if Self::compiled_capture_matches(condition, request)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                LogicalOperator::Not => {
                    if conditions.len() != 1 {
                        return Err(RequestProcessingError::RuleValidation {
                            reason: "NOT operator must have exactly one condition".to_string(),
                        }
                        .into());
                    }
                    let result = Self::compiled_capture_matches(&conditions[0], request)?;
                    Ok(!result)
                }
            },
        }
    }

    /// Check if compiled simple condition matches request
    fn compiled_simple_matches(
        condition: &CompiledCaptureCondition,
        request: &RequestInfo,
    ) -> Result<bool> {
        // Check method
        if let Some(ref method) = condition.method {
            if !method.is_empty() && !method.eq_ignore_ascii_case(&request.method) {
                return Ok(false);
            }
        }

        // Check host
        if let Some(ref host) = condition.host {
            if !host.is_empty() && !host.eq_ignore_ascii_case(&request.host) {
                return Ok(false);
            }
        }

        // Check pattern against URL
        Ok(condition.pattern.matches(&request.url))
    }
}

impl Default for RuleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::dao::request_processing_dao::types::ComplexCaptureRule;

    use super::*;
    use axum::body::Bytes;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_request(
        url: &str,
        method: &str,
        host: &str,
        headers: Option<HashMap<String, String>>,
    ) -> RequestInfo {
        RequestInfo {
            url: url.to_string(),
            method: method.to_string(),
            host: host.to_string(),
            headers: headers.unwrap_or_default(),
            body: Bytes::new(),
        }
    }

    fn create_test_capture_rule(
        capture_type: CaptureType,
        pattern: &str,
        method: Option<&str>,
        host: Option<&str>,
    ) -> CaptureRule {
        let simple_condition = SimpleCaptureCondition {
            capture_type,
            pattern: pattern.to_string(),
            method: method.map(|s| s.to_string()),
            host: host.map(|s| s.to_string()),
            config: json!({}),
        };

        CaptureRule {
            id: None,
            condition: CaptureCondition::Simple(simple_condition),
        }
    }

    fn create_test_request_rule(
        name: &str,
        priority: i32,
        enabled: bool,
        capture: CaptureRule,
    ) -> RequestRule {
        RequestRule {
            id: None,
            name: name.to_string(),
            description: None,
            enabled,
            priority,
            capture,
            handlers: vec![],
        }
    }

    #[test]
    fn test_glob_pattern_matching() {
        let matcher = RuleMatcher::new();
        let capture =
            create_test_capture_rule(CaptureType::Glob, "https://api.example.com/*", None, None);

        let request1 = create_test_request(
            "https://api.example.com/users",
            "GET",
            "api.example.com",
            None,
        );
        let request2 = create_test_request(
            "https://other.example.com/users",
            "GET",
            "other.example.com",
            None,
        );

        assert!(matcher.capture_matches(&capture, &request1).unwrap());
        assert!(!matcher.capture_matches(&capture, &request2).unwrap());
    }

    #[test]
    fn test_method_filtering() {
        let matcher = RuleMatcher::new();
        let capture = create_test_capture_rule(CaptureType::Glob, "*", Some("POST"), None);

        let request1 = create_test_request("/api/users", "POST", "api.example.com", None);
        let request2 = create_test_request("/api/users", "GET", "api.example.com", None);

        assert!(matcher.capture_matches(&capture, &request1).unwrap());
        assert!(!matcher.capture_matches(&capture, &request2).unwrap());
    }

    #[test]
    fn test_priority_sorting() {
        let matcher = RuleMatcher::new();
        let capture1 = create_test_capture_rule(CaptureType::Glob, "*", None, None);
        let capture2 = create_test_capture_rule(CaptureType::Glob, "*", None, None);

        let rule1 = create_test_request_rule("Low Priority", 10, true, capture1);
        let rule2 = create_test_request_rule("High Priority", 100, true, capture2);

        let rules = vec![rule1, rule2];
        let request = create_test_request("/api/users", "GET", "api.example.com", None);

        let matching_rules = matcher.find_matching_rules(&rules, &request).unwrap();

        assert_eq!(matching_rules.len(), 2);
        assert_eq!(matching_rules[0].name, "High Priority");
        assert_eq!(matching_rules[1].name, "Low Priority");
    }

    #[test]
    fn test_simple_rule_exact_matching() {
        let matcher = RuleMatcher::new();

        // Create a simple capture rule with exact pattern
        let capture = create_test_capture_rule(
            CaptureType::Exact,
            "https://api.example.com/users",
            Some("GET"),
            Some("api.example.com"),
        );

        let rule = create_test_request_rule("Simple Exact Rule", 50, true, capture);
        let rules = vec![rule];

        // Test exact match
        let request1 = create_test_request(
            "https://api.example.com/users",
            "GET",
            "api.example.com",
            None,
        );

        // Test non-matching URL
        let request2 = create_test_request(
            "https://api.example.com/posts",
            "GET",
            "api.example.com",
            None,
        );

        // Test non-matching method
        let request3 = create_test_request(
            "https://api.example.com/users",
            "POST",
            "api.example.com",
            None,
        );

        let matching_rules1 = matcher.find_matching_rules(&rules, &request1).unwrap();
        let matching_rules2 = matcher.find_matching_rules(&rules, &request2).unwrap();
        let matching_rules3 = matcher.find_matching_rules(&rules, &request3).unwrap();

        assert_eq!(matching_rules1.len(), 1);
        assert_eq!(matching_rules1[0].name, "Simple Exact Rule");
        assert_eq!(matching_rules2.len(), 0);
        assert_eq!(matching_rules3.len(), 0);
    }

    #[test]
    fn test_complex_rule_logical_operators() {
        let matcher = RuleMatcher::new();

        // Create simple conditions
        let simple_condition1 = SimpleCaptureCondition {
            capture_type: CaptureType::Contains,
            pattern: "api.example.com".to_string(),
            method: None,
            host: None,
            config: json!({}),
        };

        let simple_condition2 = SimpleCaptureCondition {
            capture_type: CaptureType::Contains,
            pattern: ".json".to_string(),
            method: Some("GET".to_string()),
            host: None,
            config: json!({}),
        };

        // Create complex AND condition
        let complex_condition = CaptureCondition::Complex(ComplexCaptureRule {
            operator: LogicalOperator::And,
            conditions: vec![
                CaptureCondition::Simple(simple_condition1.clone()),
                CaptureCondition::Simple(simple_condition2.clone()),
            ],
        });

        let capture_rule = CaptureRule {
            id: None,
            condition: complex_condition,
        };

        let rule = create_test_request_rule("Complex AND Rule", 75, true, capture_rule);
        let rules = vec![rule];

        // Test request that matches both conditions
        let request1 = create_test_request(
            "https://api.example.com/data.json",
            "GET",
            "api.example.com",
            None,
        );

        // Test request that matches only first condition
        let request2 = create_test_request(
            "https://api.example.com/users",
            "GET",
            "api.example.com",
            None,
        );

        // Test request that matches only second condition
        let request3 =
            create_test_request("https://example.com/data.json", "GET", "example.com", None);

        // Test request with wrong method
        let request4 = create_test_request(
            "https://api.example.com/data.json",
            "POST",
            "api.example.com",
            None,
        );

        let matching_rules1 = matcher.find_matching_rules(&rules, &request1).unwrap();
        let matching_rules2 = matcher.find_matching_rules(&rules, &request2).unwrap();
        let matching_rules3 = matcher.find_matching_rules(&rules, &request3).unwrap();
        let matching_rules4 = matcher.find_matching_rules(&rules, &request4).unwrap();

        // Debug output
        println!(
            "Request1 URL: {}, Method: {}",
            request1.url, request1.method
        );
        println!("Matching rules for request1: {}", matching_rules1.len());

        // Test individual conditions
        let condition1_match = matcher
            .evaluate_condition(
                &CaptureCondition::Simple(simple_condition1.clone()),
                &request1,
            )
            .unwrap();
        let condition2_match = matcher
            .evaluate_condition(
                &CaptureCondition::Simple(simple_condition2.clone()),
                &request1,
            )
            .unwrap();
        println!("Condition1 (api.example.com) matches: {}", condition1_match);
        println!("Condition2 (.json + GET) matches: {}", condition2_match);

        // Test the Contains pattern directly
        println!(
            "URL contains 'api.example.com': {}",
            request1.url.contains("api.example.com")
        );
        println!("URL contains '.json': {}", request1.url.contains(".json"));
        println!(
            "Method matches 'GET': {}",
            request1.method.eq_ignore_ascii_case("GET")
        );

        // Only first request should match (satisfies both AND conditions)
        assert_eq!(matching_rules1.len(), 1);
        assert_eq!(matching_rules1[0].name, "Complex AND Rule");
        assert_eq!(matching_rules2.len(), 0); // Missing pattern match for .json
        assert_eq!(matching_rules3.len(), 0); // Missing api.example.com in URL
        assert_eq!(matching_rules4.len(), 0); // Wrong method
    }

    #[test]
    fn test_complex_rule_and_with_not() {
        let matcher = RuleMatcher::new();

        // Create simple condition that matches all URLs
        let match_all_condition = SimpleCaptureCondition {
            capture_type: CaptureType::Glob,
            pattern: "*".to_string(),
            method: None,
            host: None,
            config: json!({}),
        };

        // Create condition that matches .json files (to be negated)
        let json_condition = SimpleCaptureCondition {
            capture_type: CaptureType::Contains,
            pattern: ".json".to_string(),
            method: None,
            host: None,
            config: json!({}),
        };

        // Create NOT condition for .json files
        let not_json_condition = CaptureCondition::Complex(ComplexCaptureRule {
            operator: LogicalOperator::Not,
            conditions: vec![CaptureCondition::Simple(json_condition)],
        });

        // Create complex AND condition: match all AND NOT .json
        let complex_condition = CaptureCondition::Complex(ComplexCaptureRule {
            operator: LogicalOperator::And,
            conditions: vec![
                CaptureCondition::Simple(match_all_condition),
                not_json_condition,
            ],
        });

        let capture_rule = CaptureRule {
            id: None,
            condition: complex_condition,
        };

        let rule = create_test_request_rule("AND with NOT Rule", 80, true, capture_rule);
        let rules = vec![rule];

        // Test request with .json - should NOT match
        let request1 = create_test_request(
            "https://api.example.com/data.json",
            "GET",
            "api.example.com",
            None,
        );

        // Test request without .json - should match
        let request2 = create_test_request(
            "https://api.example.com/users",
            "GET",
            "api.example.com",
            None,
        );

        // Test another request without .json - should match
        let request3 =
            create_test_request("https://example.com/posts", "POST", "example.com", None);

        // Test request with .json in middle - should NOT match
        let request4 = create_test_request(
            "https://api.example.com/data.json/extra",
            "GET",
            "api.example.com",
            None,
        );

        let matching_rules1 = matcher.find_matching_rules(&rules, &request1).unwrap();
        let matching_rules2 = matcher.find_matching_rules(&rules, &request2).unwrap();
        let matching_rules3 = matcher.find_matching_rules(&rules, &request3).unwrap();
        let matching_rules4 = matcher.find_matching_rules(&rules, &request4).unwrap();

        // Debug output
        println!("Testing AND with NOT operator:");
        println!("Request1 (.json): {} matches", matching_rules1.len());
        println!("Request2 (no .json): {} matches", matching_rules2.len());
        println!("Request3 (no .json): {} matches", matching_rules3.len());
        println!(
            "Request4 (.json in path): {} matches",
            matching_rules4.len()
        );

        // Requests with .json should NOT match (due to NOT condition)
        assert_eq!(matching_rules1.len(), 0);
        assert_eq!(matching_rules4.len(), 0);

        // Requests without .json should match
        assert_eq!(matching_rules2.len(), 1);
        assert_eq!(matching_rules2[0].name, "AND with NOT Rule");
        assert_eq!(matching_rules3.len(), 1);
        assert_eq!(matching_rules3[0].name, "AND with NOT Rule");
    }
}
