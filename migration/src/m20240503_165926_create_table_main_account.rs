use sea_orm_migration::prelude::*;
use crate::tables::{GuildConfig, MainAccount};

const FK_GUILD_ID: &str = "fk_guild_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(MainAccount::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(MainAccount::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::Name)
                        .string()
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::GuildId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::Version)
                        .tiny_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::JoinDate)
                        .date_time()
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::IsServerCreator)
                        .boolean()
                        .default(false)
                        .not_null()
                )
                .col(
                    ColumnDef::new(MainAccount::IsLeaved)
                        .boolean()
                        .default(false)
                        .not_null()
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_GUILD_ID)
                        .from_col(MainAccount::GuildId)
                        .to(GuildConfig::Table, GuildConfig::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .to_owned();

        manager
            .create_table(table)
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::drop()
                .table(MainAccount::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
