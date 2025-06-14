use sea_orm::Schema;
use sea_orm_migration::prelude::*;

use crate::entities::prelude::ApiDebug;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建api_debug表
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        let table = builder.build(schema.create_table_from_entity(ApiDebug).if_not_exists());
        manager.get_connection().execute(table).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiDebug).to_owned())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::Migrator;
    use sea_orm::{Database, EntityTrait, PaginatorTrait};
    use sea_orm_migration::MigratorTrait;

    #[tokio::test]
    async fn test_migration() {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        // 运行迁移
        Migrator::up(&db, None).await.unwrap();

        // 验证表是否存在（通过尝试查询）
        use crate::entities::api_debug::Entity;
        let count = Entity::find().count(&db).await.unwrap();
        assert_eq!(count, 0);

        // 回滚迁移
        Migrator::down(&db, None).await.unwrap();
    }
}
