use chrono::{DateTime, Utc};
use crate::utils::enums;

#[derive(sqlx::FromRow)]
pub struct Main {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) guild_id: u64,
	pub(crate) version: u8,
	pub(crate) join_date: DateTime<Utc>,
	pub(crate) is_sc: bool,
	pub(crate) is_leaved: bool
}

#[derive(sqlx::FromRow)]
pub struct Sub {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) guild_id: u64,
	pub(crate) join_date: DateTime<Utc>,
	pub(crate) main_uid: u64,
	pub(crate) first_cert: u64,
	pub(crate) second_cert: Option<u64>
}

#[derive(sqlx::FromRow)]
pub struct Confirmed {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) guild_id: u64,
	pub(crate) account_type: enums::AccountType,
	pub(crate) main_uid: Option<u64>,
	pub(crate) first_cert: Option<u64>,
	pub(crate) second_cert: Option<u64>
}

#[derive(sqlx::FromRow)]
pub struct Pending {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) guild_id: u64,
	pub(crate) account_type: enums::AccountType,
	pub(crate) message_id: u64,
	pub(crate) end_voting: Option<DateTime<Utc>>,
	pub(crate) main_uid: Option<u64>,
	pub(crate) first_cert: Option<u64>
}
