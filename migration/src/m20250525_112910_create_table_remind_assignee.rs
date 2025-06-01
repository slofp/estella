use sea_orm_migration::prelude::*;

use crate::tables::{RemindAssignee, UserData};

const FK_USER_ID: &str = "assignee_fk_user_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let table = Table::create()
			.table(RemindAssignee::Table)
			.if_not_exists()
			.col(
				ColumnDef::new(RemindAssignee::Id)
					.unsigned()
					.primary_key()
					.not_null()
					.auto_increment(),
			)
			.col(ColumnDef::new(RemindAssignee::UserId).big_unsigned().not_null())
			.foreign_key(
				ForeignKey::create()
					.name(FK_USER_ID)
					.from_col(RemindAssignee::UserId)
					.to(UserData::Table, UserData::Uid)
					.on_delete(ForeignKeyAction::Cascade)
					.on_update(ForeignKeyAction::Cascade),
			)
			.to_owned();

		manager.create_table(table).await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let table = Table::drop().table(RemindAssignee::Table).to_owned();

		manager.drop_table(table).await
	}
}
