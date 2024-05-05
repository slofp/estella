use sea_orm_migration::prelude::*;
use crate::tables::UserData;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(UserData::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(UserData::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(UserData::Glacialeur)
                        .string_len(14)
                        .null()
                )
                .to_owned();

        manager
            .create_table(table)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::drop()
                .table(UserData::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
