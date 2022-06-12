#[derive(sqlx::FromRow)]
pub struct Data {
	pub(crate) uid: u64,
	pub(crate) glacialeur: Option<String>
}
