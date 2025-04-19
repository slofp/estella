use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigData {
	token: String,
	bot_id: u64,
	owner_id: u64,

	db_url: String,
	db_username: String,
	db_password: String,
	db_database: String,
}

impl ConfigData {
	pub fn get_token(&self) -> &String {
		&self.token
	}

	pub fn get_bot_id(&self) -> &u64 {
		&self.bot_id
	}

	pub fn get_owner_id(&self) -> &u64 {
		&self.owner_id
	}

	pub fn get_db_url(&self) -> String {
		format!(
			"mysql://{}:{}@{}/{}",
			self.db_username, self.db_password, self.db_url, self.db_database
		)
	}
}
