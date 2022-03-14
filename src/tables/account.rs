use chrono::{DateTime, Utc};

pub struct Main {
	uid: u64,
	name: String,
	version: u8,
	join_date: DateTime<Utc>,
	is_sc: bool,
	is_leaved: bool
}

pub struct Sub {
	uid: u64,
	name: String,
	join_date: DateTime<Utc>,
	main_uid: u64,
	first_cert: u64,
	second_cert: Option<u64>
}

pub struct Pending {
	uid: u64,
	end_voting: DateTime<Utc>
}
