use sea_orm_migration::prelude::*;
use crate::tables::GuildConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(GuildConfig::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(GuildConfig::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(GuildConfig::WhiteList)
                        .boolean()
                        .default(false)
                        .not_null()
                )
                .col(
                    ColumnDef::new(GuildConfig::LeaveBan)
                        .boolean()
                        .default(false)
                        .not_null()
                )
                .col(
                    ColumnDef::new(GuildConfig::LogChannelId)
                        .big_unsigned()
                        .null()
                )
                .col(
                    ColumnDef::new(GuildConfig::AuthRoleId)
                        .big_unsigned()
                        .null()
                )
                .col(
                    ColumnDef::new(GuildConfig::BotRoleId)
                        .big_unsigned()
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
                .table(GuildConfig::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
