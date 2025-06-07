use sea_orm_migration::prelude::*;

use crate::tables::{TalkHistory, UserData};

const FK_USER_ID: &str = "talk_history_fk_user_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
			.table(TalkHistory::Table)
			.if_not_exists()
			.col(
				ColumnDef::new(TalkHistory::Id)
					.unsigned()
					.primary_key()
					.not_null()
					.auto_increment(),
			)
            .col(
				ColumnDef::new(TalkHistory::UserId)
					.big_unsigned()
					.not_null(),
			)
			.col(
				ColumnDef::new(TalkHistory::ChatId)
					.text()
					.not_null(),
			)
			.col(
				ColumnDef::new(TalkHistory::InputText)
					.text()
					.not_null(),
			)
			.col(
				ColumnDef::new(TalkHistory::OutputText)
					.text()
					.not_null(),
			)
			.col(
				ColumnDef::new(TalkHistory::TalkDate)
					.date_time()
					.not_null(),
			)
            .foreign_key(
				ForeignKey::create()
					.name(FK_USER_ID)
					.from_col(TalkHistory::UserId)
					.to(UserData::Table, UserData::Uid)
					.on_delete(ForeignKeyAction::Cascade)
					.on_update(ForeignKeyAction::Cascade),
			)
			.to_owned();

		manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(TalkHistory::Table).to_owned();

		manager.drop_table(table).await
    }
}
