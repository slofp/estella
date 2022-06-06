#[derive(sqlx::FromRow)]
pub struct Config {
	pub(crate) uid: u64,
	pub(crate) white_list: bool,
	pub(crate) leave_ban: bool,
	pub(crate) log_channel_id: Option<u64>
}
