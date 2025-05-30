pub mod common;
pub mod error;
pub mod handlers;
pub mod matcher;
pub mod request_info;
pub mod response_info;
pub mod types;
pub mod validator;

pub use common::{BodyUtils, HeaderUtils, HttpMessage};
pub use error::RequestProcessingError;
pub use handlers::HandlerRule;
pub use matcher::RuleMatcher;
pub use request_info::RequestInfo;
pub use response_info::ResponseInfo;
pub use types::{CaptureRule, LocalFileConfig, ModifyRequestConfig, RequestRule};
pub use validator::RuleValidator;

use crate::entities::{
    capture::{self, Entity as CaptureEntity},
    handler::{self, ActiveModel as HandlerActiveModel, Entity as HandlerEntity},
    rule::{self, ActiveModel as RuleActiveModel, Entity as RuleEntity},
};
use anyhow::{Result, anyhow};
use sea_orm::*;
use std::sync::Arc;

pub struct RequestProcessingDao {
    db: Arc<DatabaseConnection>,
}

impl RequestProcessingDao {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new request processing rule
    pub async fn create_rule(&self, rule: RequestRule) -> Result<i32> {
        let txn = self.db.begin().await?;

        // Insert rule
        let rule_active_model = RuleActiveModel {
            id: NotSet,
            name: Set(rule.name),
            description: Set(rule.description),
            enabled: Set(rule.enabled),
            priority: Set(rule.priority),
            ..Default::default()
        };

        let rule_result = rule_active_model.insert(&txn).await?;
        let rule_id = rule_result.id;

        // Insert capture using the conversion method
        let capture = rule.capture;
        let capture_active_model = capture::Model::from_capture_rule(&capture, rule_id)
            .map_err(|e| anyhow!("Failed to convert capture rule: {}", e))?;
        capture_active_model.insert(&txn).await?;

        // Insert handlers
        for handler in rule.handlers {
            let handler_active_model = HandlerActiveModel {
                id: NotSet,
                rule_id: Set(rule_id),
                handler_type: Set(handler.handler_type),
                name: Set(handler.name),
                description: Set(handler.description),
                execution_order: Set(handler.execution_order),
                config: Set(handler.config),
                enabled: Set(handler.enabled),
                created_at: NotSet,
                updated_at: NotSet,
            };
            handler_active_model.insert(&txn).await?;
        }

        txn.commit().await?;
        Ok(rule_id)
    }

    /// Get rule by ID with capture and handlers
    pub async fn get_rule(&self, rule_id: i32) -> Result<Option<RequestRule>> {
        let rule = RuleEntity::find_by_id(rule_id)
            .one(self.db.as_ref())
            .await?;

        if let Some(rule) = rule {
            let capture = CaptureEntity::find()
                .filter(capture::Column::RuleId.eq(rule_id))
                .one(self.db.as_ref())
                .await?;

            let handlers = HandlerEntity::find()
                .filter(handler::Column::RuleId.eq(rule_id))
                .order_by_asc(handler::Column::ExecutionOrder)
                .all(self.db.as_ref())
                .await?;

            // Since capture is now required, return None if no capture found
            if let Some(capture) = capture {
                let capture_rule = capture
                    .to_capture_rule()
                    .map_err(|e| anyhow!("Failed to convert capture: {}", e))?;

                Ok(Some(RequestRule {
                    id: Some(rule.id),
                    name: rule.name,
                    description: rule.description,
                    enabled: rule.enabled,
                    priority: rule.priority,
                    capture: capture_rule,
                    handlers: handlers
                        .into_iter()
                        .map(|h| HandlerRule {
                            id: Some(h.id),
                            handler_type: h.handler_type,
                            name: h.name,
                            description: h.description,
                            execution_order: h.execution_order,
                            config: h.config,
                            enabled: h.enabled,
                        })
                        .collect(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get all rules ordered by priority
    pub async fn list_rules(&self) -> Result<Vec<RequestRule>> {
        let rules = RuleEntity::find()
            .order_by_desc(rule::Column::Priority)
            .order_by_asc(rule::Column::Id)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for rule in rules {
            if let Some(full_rule) = self.get_rule(rule.id).await? {
                result.push(full_rule);
            }
        }

        Ok(result)
    }

    /// Update a rule
    pub async fn update_rule(&self, rule: RequestRule) -> Result<()> {
        let rule_id = rule
            .id
            .ok_or_else(|| anyhow!("Rule ID is required for update"))?;

        let txn = self.db.begin().await?;

        // Update rule
        let rule_active_model = RuleActiveModel {
            id: Set(rule_id),
            name: Set(rule.name),
            description: Set(rule.description),
            enabled: Set(rule.enabled),
            priority: Set(rule.priority),
            updated_at: NotSet,
            ..Default::default()
        };
        rule_active_model.update(&txn).await?;

        // Delete existing captures and handlers
        CaptureEntity::delete_many()
            .filter(capture::Column::RuleId.eq(rule_id))
            .exec(&txn)
            .await?;

        HandlerEntity::delete_many()
            .filter(handler::Column::RuleId.eq(rule_id))
            .exec(&txn)
            .await?;

        // Insert new capture using conversion method
        let capture = rule.capture;
        let capture_active_model = capture::Model::from_capture_rule(&capture, rule_id)
            .map_err(|e| anyhow!("Failed to convert capture rule: {}", e))?;
        CaptureEntity::insert(capture_active_model)
            .exec(&txn)
            .await?;

        // Insert new handlers
        for handler in rule.handlers {
            let handler_active_model = HandlerActiveModel {
                id: NotSet,
                rule_id: Set(rule_id),
                handler_type: Set(handler.handler_type),
                name: Set(handler.name),
                description: Set(handler.description),
                execution_order: Set(handler.execution_order),
                config: Set(handler.config),
                enabled: Set(handler.enabled),
                created_at: NotSet,
                updated_at: NotSet,
            };
            HandlerEntity::insert(handler_active_model)
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(())
    }

    /// Delete a rule and all its captures and handlers
    pub async fn delete_rule(&self, rule_id: i32) -> Result<()> {
        let txn = self.db.begin().await?;

        // Delete captures
        CaptureEntity::delete_many()
            .filter(capture::Column::RuleId.eq(rule_id))
            .exec(&txn)
            .await?;

        // Delete handlers
        HandlerEntity::delete_many()
            .filter(handler::Column::RuleId.eq(rule_id))
            .exec(&txn)
            .await?;

        // Delete rule
        RuleEntity::delete_by_id(rule_id).exec(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    /// Find matching rules for a request
    pub async fn find_matching_rules(&self, request: &RequestInfo) -> Result<Vec<RequestRule>> {
        let all_rules = self.list_rules().await?;
        let matcher = RuleMatcher::new();
        matcher.find_matching_rules(&all_rules, request)
    }

    /// Get template handlers (rule_id = 0)
    pub async fn get_template_handlers(&self) -> Result<Vec<HandlerRule>> {
        let handlers = HandlerEntity::find()
            .filter(handler::Column::RuleId.eq(0))
            .order_by_asc(handler::Column::ExecutionOrder)
            .all(self.db.as_ref())
            .await?;

        Ok(handlers
            .into_iter()
            .map(|h| HandlerRule {
                id: Some(h.id),
                handler_type: h.handler_type,
                name: h.name,
                description: h.description,
                execution_order: h.execution_order,
                config: h.config,
                enabled: h.enabled,
            })
            .collect())
    }

    /// Enable or disable a rule
    pub async fn toggle_rule(&self, rule_id: i32, enabled: bool) -> Result<()> {
        let rule_active_model = RuleActiveModel {
            id: Set(rule_id),
            enabled: Set(enabled),
            ..Default::default()
        };
        rule_active_model
            .update(self.db.as_ref())
            .await?;
        Ok(())
    }
}
