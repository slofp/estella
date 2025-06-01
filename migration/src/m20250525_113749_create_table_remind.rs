use sea_orm_migration::{prelude::*};

use crate::tables::{Remind, RemindAssignee, UserData};

const FK_AUTHOR_ID: &str = "remind_fk_author_id";
const FK_ASSIGNEE_ID: &str = "remind_fk_assignee_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
			.table(Remind::Table)
			.if_not_exists()
			.col(
				ColumnDef::new(Remind::Id)
					.unsigned()
					.primary_key()
					.not_null()
					.auto_increment(),
			)
            .col(
				ColumnDef::new(Remind::TaskName)
					.text()
					.not_null()
                    .default(""),
			)
            .col(
				ColumnDef::new(Remind::AuthorId)
					.big_unsigned()
					.not_null(),
			)
            .col(
				ColumnDef::new(Remind::AssigneesId)
					.unsigned(),
			)
            .col(
				ColumnDef::new(Remind::RemindDate)
                    .date_time()
					.not_null(),
			)
			.foreign_key(
				ForeignKey::create()
					.name(FK_AUTHOR_ID)
					.from_col(Remind::AuthorId)
					.to(UserData::Table, UserData::Uid)
					.on_delete(ForeignKeyAction::Cascade)
					.on_update(ForeignKeyAction::Cascade),
			)
            .foreign_key(
				ForeignKey::create()
					.name(FK_ASSIGNEE_ID)
					.from_col(Remind::AssigneesId)
					.to(RemindAssignee::Table, RemindAssignee::Id)
					.on_delete(ForeignKeyAction::Cascade)
					.on_update(ForeignKeyAction::Cascade),
			)
			.to_owned();

		manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::drop().table(Remind::Table).to_owned();

		manager.drop_table(table).await
    }
}
