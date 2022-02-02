use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigData {
	token: String,

	db_url: String,
	db_username: String,
	db_password: String,
	db_database: String
}

impl ConfigData {
	pub fn get_token(&self) -> String {
		token
	}

	pub fn get_db_url(&self) -> String {
		format!("mysql://")
	}
}
