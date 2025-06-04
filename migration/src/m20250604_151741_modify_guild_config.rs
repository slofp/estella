use sea_orm_migration::prelude::*;

use crate::tables::GuildConfig;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::alter()
                .table(GuildConfig::Table)
                .add_column(
                    ColumnDef::new(GuildConfig::SendAiChatChannelId)
                        .big_unsigned()
                        .null()
                )
                .to_owned();

        manager
            .alter_table(table)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::alter()
                .table(GuildConfig::Table)
                .drop_column(GuildConfig::SendAiChatChannelId)
                .to_owned();

        manager
            .alter_table(table)
            .await
    }
}
