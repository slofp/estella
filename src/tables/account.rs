use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct Main {
	uid: u64,
	name: String,
	version: u8,
	join_date: DateTime<Utc>,
	is_sc: bool,
	is_leaved: bool
}

#[derive(sqlx::FromRow)]
pub struct Sub {
	uid: u64,
	name: String,
	join_date: DateTime<Utc>,
	main_uid: u64,
	first_cert: u64,
	second_cert: Option<u64>
}

#[derive(sqlx::FromRow)]
pub struct Pending {
	uid: u64,
	end_voting: DateTime<Utc>
}
