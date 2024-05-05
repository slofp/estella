use std::fmt::Display;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u8", db_type = "TinyUnsigned")]
#[repr(u8)]
pub enum AccountType {
	Main = 1,
	Sub = 2,
}

impl Display for AccountType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			AccountType::Main => "メイン",
			AccountType::Sub => "サブ"
		}.to_string())
	}
}
