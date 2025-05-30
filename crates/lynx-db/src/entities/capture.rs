//! Capture Entity for defining capture rules

use async_trait::async_trait;
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;

/// Capture type enumeration
#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[serde(rename_all = "camelCase")]
pub enum CaptureType {
    #[sea_orm(string_value = "glob")]
    Glob,
    #[sea_orm(string_value = "regex")]
    Regex,
    #[sea_orm(string_value = "exact")]
    Exact,
    #[sea_orm(string_value = "contains")]
    Contains,
}

impl Default for CaptureType {
    fn default() -> Self {
        Self::Glob
    }
}

/// Rule type enumeration
#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[serde(rename_all = "camelCase")]
pub enum RuleType {
    #[sea_orm(string_value = "simple")]
    Simple,
    #[sea_orm(string_value = "complex")]
    Complex,
}

impl Default for RuleType {
    fn default() -> Self {
        Self::Simple
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "capture")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub rule_id: i32,
    pub rule_type: RuleType,
    #[sea_orm(column_type = "Json", nullable)]
    pub simple_config: Option<JsonValue>,
    #[sea_orm(column_type = "Json", nullable)]
    pub complex_condition: Option<JsonValue>,
    #[serde(skip)]
    pub created_at: i64,
    #[serde(skip)]
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::rule::Entity",
        from = "Column::RuleId",
        to = "super::rule::Column::Id"
    )]
    Rule,
}

impl Related<super::rule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}

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

impl Model {
    /// Convert to CaptureRule type
    pub fn to_capture_rule(
        &self,
    ) -> Result<crate::dao::request_processing_dao::types::CaptureRule, Box<dyn std::error::Error>>
    {
        use crate::dao::request_processing_dao::types::{
            CaptureCondition, CaptureRule, SimpleCaptureCondition,
        };

        let condition = match self.rule_type {
            RuleType::Simple => {
                if let Some(simple_json) = &self.simple_config {
                    let simple: SimpleCaptureCondition =
                        serde_json::from_value(simple_json.clone())?;
                    CaptureCondition::Simple(simple)
                } else {
                    return Err("Simple rule type but no simple config found".into());
                }
            }
            RuleType::Complex => {
                if let Some(complex_json) = &self.complex_condition {
                    serde_json::from_value(complex_json.clone())?
                } else {
                    return Err("Complex rule type but no complex condition found".into());
                }
            }
        };

        Ok(CaptureRule {
            id: Some(self.id),
            condition,
        })
    }

    /// Create from CaptureRule type
    pub fn from_capture_rule(
        rule: &crate::dao::request_processing_dao::types::CaptureRule,
        rule_id: i32,
    ) -> Result<ActiveModel, Box<dyn std::error::Error>> {
        use crate::dao::request_processing_dao::types::CaptureCondition;
        use sea_orm::ActiveValue::*;

        let (rule_type, simple_config, complex_condition) = match &rule.condition {
            CaptureCondition::Simple(simple) => {
                (RuleType::Simple, Some(serde_json::to_value(simple)?), None)
            }
            CaptureCondition::Complex(_) => (
                RuleType::Complex,
                None,
                Some(serde_json::to_value(&rule.condition)?),
            ),
        };

        Ok(ActiveModel {
            id: if let Some(id) = rule.id {
                Set(id)
            } else {
                NotSet
            },
            rule_id: Set(rule_id),
            rule_type: Set(rule_type),
            simple_config: Set(simple_config),
            complex_condition: Set(complex_condition),
            created_at: NotSet,
            updated_at: NotSet,
        })
    }
}
