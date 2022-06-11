#[derive(sqlx::FromRow)]
pub struct Data {
	uid: u64,
	glacialeur: Option<String>
}
