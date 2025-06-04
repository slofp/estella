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
                .add_column(
                    ColumnDef::new(UserData::CallName)
                        .string_len(32)
                        .null()
                        .default("")
                )
                .add_column(
                    ColumnDef::new(UserData::Gender)
                        .char_len(1)
                        .null()
                )
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::alter()
                .table(UserData::Table)
                .drop_column(UserData::CallName)
                .drop_column(UserData::Gender)
                .drop_column(UserData::LikabilityLevel)
                .to_owned();

        manager
            .alter_table(table)
            .await
    }
}
