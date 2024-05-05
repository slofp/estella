use sea_orm_migration::prelude::*;
use crate::tables::{PendingAccount, GuildConfig, MainAccount};

const FK_MAIN_UID: &str = "pending_fk_main_uid";
const FK_FIRST_CERT: &str = "pending_fk_first_cert";
const FK_GUILD_ID: &str = "pending_fk_guild_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(PendingAccount::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(PendingAccount::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(PendingAccount::Name)
                        .string()
                        .null()
                )
                .col(
                    ColumnDef::new(PendingAccount::GuildId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(PendingAccount::AccountType)
                        .tiny_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(PendingAccount::MessageId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(PendingAccount::EndVoting)
                        .date_time()
                        .null()
                )
                .col(
                    ColumnDef::new(PendingAccount::MainUid)
                        .big_unsigned()
                        .null()
                )
                .col(
                    ColumnDef::new(PendingAccount::FirstCert)
                        .big_unsigned()
                        .null()
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_GUILD_ID)
                        .from_col(PendingAccount::GuildId)
                        .to(GuildConfig::Table, GuildConfig::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_MAIN_UID)
                        .from_col(PendingAccount::MainUid)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_FIRST_CERT)
                        .from_col(PendingAccount::FirstCert)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::SetNull)
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
                .table(PendingAccount::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
