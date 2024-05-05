use sea_orm_migration::prelude::*;
use crate::tables::{GuildConfig, MainAccount, SubAccount};

const FK_MAIN_UID: &str = "sub_fk_main_uid";
const FK_FIRST_CERT: &str = "sub_fk_first_cert";
const FK_SECOND_CERT: &str = "sub_fk_second_cert";
const FK_GUILD_ID: &str = "sub_fk_guild_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table =
            Table::create()
                .table(SubAccount::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(SubAccount::Uid)
                        .big_unsigned()
                        .primary_key()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::Name)
                        .string()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::GuildId)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::JoinDate)
                        .date_time()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::MainUid)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::FirstCert)
                        .big_unsigned()
                        .not_null()
                )
                .col(
                    ColumnDef::new(SubAccount::SecondCert)
                        .big_unsigned()
                        .null()
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_GUILD_ID)
                        .from_col(SubAccount::GuildId)
                        .to(GuildConfig::Table, GuildConfig::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_MAIN_UID)
                        .from_col(SubAccount::MainUid)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_FIRST_CERT)
                        .from_col(SubAccount::FirstCert)
                        .to(MainAccount::Table, MainAccount::Uid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .foreign_key(
                    ForeignKey::create()
                        .name(FK_SECOND_CERT)
                        .from_col(SubAccount::SecondCert)
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
                .table(SubAccount::Table)
                .to_owned();

        manager
            .drop_table(table)
            .await
    }
}
