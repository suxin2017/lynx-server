use crate::entities::app_config::{self, ActiveModel, Entity};
use anyhow::Result;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum RecordingStatus {
    #[default]
    StartRecording,
    PauseRecording,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CaptureSwitch {
    pub recording_status: RecordingStatus,
}

impl From<CaptureSwitch> for ActiveModel {
    fn from(switch: CaptureSwitch) -> Self {
        ActiveModel {
            id: NotSet,
            key: Set("capture_switch".to_string()),
            value: Set(json!({
                "recordingStatus": switch.recording_status,
            })),
            description: Set(Some("Capture switch configuration".to_string())),
        }
    }
}

pub struct CaptureSwitchDao {
    db: Arc<DatabaseConnection>,
}

impl CaptureSwitchDao {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn get_capture_switch(&self) -> Result<CaptureSwitch> {
        let config = Entity::find()
            .filter(app_config::Column::Key.eq("capture_switch"))
            .one(self.db.as_ref())
            .await?;

        match config {
            Some(model) => {
                let value = model.value.get("recordingStatus").ok_or_else(|| {
                    anyhow::anyhow!("missing field `recordingStatus` in config value")
                })?;
                Ok(CaptureSwitch {
                    recording_status: serde_json::from_value(value.clone())?,
                })
            }
            None => Ok(CaptureSwitch::default()),
        }
    }

    pub async fn update_capture_switch(&self, switch: CaptureSwitch) -> Result<()> {
        let existing = Entity::find()
            .filter(app_config::Column::Key.eq("capture_switch"))
            .one(self.db.as_ref())
            .await?;

        match existing {
            Some(model) => {
                let mut update: ActiveModel = model.into();
                update.value = Set(json!({
                    "recordingStatus": switch.recording_status,
                }));
                Entity::update(update).exec(self.db.as_ref()).await?;
            }
            None => {
                let model: ActiveModel = switch.into();
                Entity::insert(model).exec(self.db.as_ref()).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::Migrator;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_capture_switch_default() -> Result<()> {
        let db = setup_test_db().await;
        let dao = CaptureSwitchDao::new(Arc::new(db));

        // 测试获取默认值
        let switch = dao.get_capture_switch().await?;
        assert!(matches!(
            switch.recording_status,
            RecordingStatus::StartRecording
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_capture_switch_crud() -> Result<()> {
        let db = setup_test_db().await;
        let dao = CaptureSwitchDao::new(Arc::new(db));

        // 测试更新状态
        let mut switch = CaptureSwitch {
            recording_status: RecordingStatus::StartRecording,
        };
        dao.update_capture_switch(switch.clone()).await?;

        // 验证更新
        let loaded_switch = dao.get_capture_switch().await?;
        assert!(matches!(
            loaded_switch.recording_status,
            RecordingStatus::StartRecording
        ));

        // 测试切换状态
        switch.recording_status = RecordingStatus::PauseRecording;
        dao.update_capture_switch(switch).await?;

        // 验证切换
        let loaded_switch = dao.get_capture_switch().await?;
        assert!(matches!(
            loaded_switch.recording_status,
            RecordingStatus::PauseRecording
        ));

        Ok(())
    }
}
