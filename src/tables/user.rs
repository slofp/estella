#[derive(sqlx::FromRow)]
pub struct Data {
	uid: u64,
	username: String,
	discriminator: u16,
	glacialeur: Option<String>
}

#[derive(sqlx::FromRow)]
pub struct Level {
	uid: u64,
	level: u64,
	exp: f64
}
