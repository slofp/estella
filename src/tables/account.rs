use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct Main {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) version: u8,
	pub(crate) join_date: DateTime<Utc>,
	pub(crate) is_sc: bool,
	pub(crate) is_leaved: bool
}

#[derive(sqlx::FromRow)]
pub struct Sub {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) join_date: DateTime<Utc>,
	pub(crate) main_uid: u64,
	pub(crate) first_cert: u64,
	pub(crate) second_cert: Option<u64>
}

#[derive(sqlx::FromRow)]
pub struct Pending {
	pub(crate) uid: u64,
	pub(crate) name: String,
	pub(crate) end_voting: DateTime<Utc>
}
