use sea_orm_migration::prelude::*;
use crate::tables::{ConfirmedAccount, GuildConfig, MainAccount};

const FK_MAIN_UID: &str = "confirm_fk_main_uid";
const FK_FIRST_CERT: &str = "confirm_fk_first_cert";
const FK_SECOND_CERT: &str = "confirm_fk_second_cert";
const FK_GUILD_ID: &str = "confirm_fk_guild_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(ConfirmedAccount::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ConfirmedAccount::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::Name)
                        .string()
                        .not_null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::GuildId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::AccountType)
                        .tiny_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::MainUid)
                        .big_unsigned()
                        .null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::FirstCert)
                        .big_unsigned()
                        .null()
                )
                .col(
                    ColumnDef::new(ConfirmedAccount::SecondCert)
                        .big_unsigned()
                        .null()
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_GUILD_ID)
                        .from_col(ConfirmedAccount::GuildId)
                        .to(GuildConfig::Table, GuildConfig::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_MAIN_UID)
                        .from_col(ConfirmedAccount::MainUid)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_FIRST_CERT)
                        .from_col(ConfirmedAccount::FirstCert)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::SetNull)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_SECOND_CERT)
                        .from_col(ConfirmedAccount::SecondCert)
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
                .table(ConfirmedAccount::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
