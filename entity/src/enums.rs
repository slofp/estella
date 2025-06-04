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

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
pub enum Gender {
	#[sea_orm(string_value = "M")]
	Male,
	#[sea_orm(string_value = "L")]
	Ladies,
}

impl Display for Gender {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Gender::Male => write!(f, "M"),
			Gender::Ladies => write!(f, "L"),
		}
	}
}
