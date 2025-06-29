//! Handler Entity for request processing actions

use async_trait::async_trait;
use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;

use crate::dao::request_processing_dao::handlers::handler_rule::HandlerRuleType;

/// Handler type enumeration
#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(30))")]
#[serde(rename_all = "camelCase")]
pub enum HandlerType {
    #[sea_orm(string_value = "block")]
    Block,
    #[sea_orm(string_value = "modify_request")]
    ModifyRequest,
    #[sea_orm(string_value = "local_file")]
    LocalFile,
    #[sea_orm(string_value = "modify_response")]
    ModifyResponse,
    #[sea_orm(string_value = "proxy_forward")]
    ProxyForward,
    #[sea_orm(string_value = "html_script_injector")]
    HtmlScriptInjector,
    #[sea_orm(string_value = "delay")]
    Delay,
}

impl From<&HandlerRuleType> for HandlerType {
    fn from(handler_rule_type: &HandlerRuleType) -> Self {
        match handler_rule_type {
            HandlerRuleType::Block(_) => Self::Block,
            HandlerRuleType::ModifyRequest(_) => Self::ModifyRequest,
            HandlerRuleType::LocalFile(_) => Self::LocalFile,
            HandlerRuleType::ModifyResponse(_) => Self::ModifyResponse,
            HandlerRuleType::ProxyForward(_) => Self::ProxyForward,
            HandlerRuleType::HtmlScriptInjector(_) => Self::HtmlScriptInjector,
            HandlerRuleType::Delay(_) => Self::Delay,
        }
    }
}

impl From<&HandlerRuleType> for JsonValue {
    fn from(handler_rule_type: &HandlerRuleType) -> Self {
        match handler_rule_type {
            HandlerRuleType::Block(config) => serde_json::to_value(config).unwrap_or_default(),
            HandlerRuleType::ModifyRequest(config) => {
                serde_json::to_value(config).unwrap_or_default()
            }
            HandlerRuleType::LocalFile(config) => serde_json::to_value(config).unwrap_or_default(),
            HandlerRuleType::ModifyResponse(config) => {
                serde_json::to_value(config).unwrap_or_default()
            }
            HandlerRuleType::ProxyForward(config) => {
                serde_json::to_value(config).unwrap_or_default()
            }
            HandlerRuleType::HtmlScriptInjector(config) => {
                serde_json::to_value(config).unwrap_or_default()
            }
            HandlerRuleType::Delay(config) => {
                serde_json::to_value(config).unwrap_or_default()
            }
        }
    }
}
impl Default for HandlerType {
    fn default() -> Self {
        Self::Block
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "handler")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub rule_id: Option<i32>,
    pub handler_type: HandlerType,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(default_value = 0)]
    pub execution_order: i32,
    #[sea_orm(column_type = "Json")]
    pub config: JsonValue,
    #[sea_orm(default_value = true)]
    pub enabled: bool,
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
