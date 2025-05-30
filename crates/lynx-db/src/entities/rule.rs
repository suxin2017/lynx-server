//! Rule Entity for capturing and processing requests

use async_trait::async_trait;
use sea_orm::{Set, entity::prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "rule")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Rule name for identification
    #[sea_orm(column_type = "Text")]
    pub name: String,

    /// Rule description
    pub description: Option<String>,

    /// Whether the rule is enabled
    #[sea_orm(default_value = true)]
    pub enabled: bool,

    /// Rule priority (higher number = higher priority)
    #[sea_orm(default_value = 0)]
    pub priority: i32,

    /// Creation timestamp
    #[serde(skip)]
    pub created_at: i64,

    /// Update timestamp
    #[serde(skip)]
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::capture::Entity")]
    Capture,
    #[sea_orm(has_many = "super::handler::Entity")]
    Handlers,
}

impl Related<super::capture::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Capture.def()
    }
}

impl Related<super::handler::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Handlers.def()
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
