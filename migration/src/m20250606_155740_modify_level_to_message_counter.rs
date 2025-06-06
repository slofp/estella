use sea_orm_migration::prelude::*;

use crate::tables::UserData;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::LikabilityLevel)
                .add_column(
                    ColumnDef::new(UserData::ChatMessageCount)
                        .unsigned()
                        .null()
                        .default(0)
                )
                .to_owned();

        manager
            .alter_table(table)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::ChatMessageCount)
                .add_column(
                    ColumnDef::new(UserData::LikabilityLevel)
                        .unsigned()
                        .null()
                        .default(0)
                )
                .to_owned();

        manager
            .alter_table(table)
            .await
    }
}
