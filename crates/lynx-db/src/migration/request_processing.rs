use sea_orm::Schema;
use sea_orm::Statement;
use sea_orm_migration::prelude::*;

use crate::dao::request_processing_dao::handlers::handler_rule::HandlerRule;
use crate::entities::handler;
use crate::entities::prelude::{Capture, Handler, Rule};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);

        // Create rule table
        let rule_table = builder.build(schema.create_table_from_entity(Rule).if_not_exists());
        manager.get_connection().execute(rule_table).await?;

        // Create capture table
        let capture_table = builder.build(schema.create_table_from_entity(Capture).if_not_exists());
        manager.get_connection().execute(capture_table).await?;

        // Create handler table
        let handler_table = builder.build(schema.create_table_from_entity(Handler).if_not_exists());
        manager.get_connection().execute(handler_table).await?;

        // 临时禁用外键约束检查
        let disable_fk = Statement::from_string(
            manager.get_database_backend(),
            "PRAGMA foreign_keys = OFF".to_owned(),
        );
        manager.get_connection().execute(disable_fk).await?;

        // Insert default handler templates
        let default_templates = HandlerRule::default_templates();
        for template in default_templates {
            let now = chrono::Utc::now().timestamp();
            let insert_stmt = Query::insert()
                .into_table(handler::Entity)
                .columns([
                    handler::Column::RuleId,
                    handler::Column::HandlerType,
                    handler::Column::Name,
                    handler::Column::Description,
                    handler::Column::ExecutionOrder,
                    handler::Column::Config,
                    handler::Column::Enabled,
                    handler::Column::CreatedAt,
                    handler::Column::UpdatedAt,
                ])
                .values_panic([
                    0.into(), // rule_id 设为 0，表示这是模板
                    template.handler_type.into(),
                    template.name.into(),
                    template.description.into(),
                    template.execution_order.into(),
                    template.config.into(),
                    template.enabled.into(),
                    now.into(),
                    now.into(),
                ])
                .to_owned();

            manager
                .get_connection()
                .execute(builder.build(&insert_stmt))
                .await?;
        }

        // 重新启用外键约束检查
        let enable_fk = Statement::from_string(
            manager.get_database_backend(),
            "PRAGMA foreign_keys = ON".to_owned(),
        );
        manager.get_connection().execute(enable_fk).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign key constraints
        manager
            .drop_table(Table::drop().table(HandlerTable::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CaptureTable::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RuleTable::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RuleTable {
    Table,
}

#[derive(DeriveIden)]
enum CaptureTable {
    Table,
}

#[derive(DeriveIden)]
enum HandlerTable {
    Table,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::handler;
    use crate::migration::Migrator;
    use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
    use sea_orm_migration::MigratorTrait;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_migration_up() {
        let _db = setup_test_db().await;
        // Migration successful if no panic occurs
    }

    #[tokio::test]
    async fn test_default_templates_insertion() {
        let db = setup_test_db().await;

        // Query all templates (records with rule_id = 0)
        let templates = handler::Entity::find()
            .filter(handler::Column::RuleId.eq(0))
            .all(&db)
            .await
            .unwrap();

        // Get expected default template count
        let expected_templates = HandlerRule::default_templates();

        // Verify the number of inserted templates is correct
        assert_eq!(
            templates.len(),
            expected_templates.len(),
            "Number of inserted templates should match default templates count"
        );

        // Verify basic attributes of each template
        for template in templates.iter() {
            assert_eq!(template.rule_id, 0, "Template rule_id should be 0");
            assert!(!template.enabled, "All default templates should be enabled");
            assert!(
                !template.name.is_empty(),
                "Template name should not be empty"
            );
            assert!(
                template.description.is_some(),
                "Template description should not be empty"
            );
            assert!(
                template.config.is_object(),
                "Template config should not be empty"
            );
        }
    }
}
