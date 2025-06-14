//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.12

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "main_account")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub uid: u64,
	pub name: String,
	pub guild_id: u64,
	pub version: u8,
	pub join_date: ChronoDateTimeUtc,
	pub is_server_creator: bool,
	pub is_leaved: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::guild_config::Entity",
		from = "Column::GuildId",
		to = "super::guild_config::Column::Uid",
		on_update = "Cascade",
		on_delete = "Cascade"
	)]
	GuildConfig,
	#[sea_orm(
        has_many = "super::sub_account::Entity"
    )]
    SubAccount,
}

impl Related<super::guild_config::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::GuildConfig.def()
	}
}

impl Related<super::sub_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
