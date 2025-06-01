use crate::ConfigData;
use sea_orm::DatabaseConnection;
use serenity::all::ShardManager;
use std::sync::Arc;

pub struct Components {
	config: Option<ConfigData>,
	mysql_client: Option<DatabaseConnection>,
	cloned_shard_manager: Option<Arc<ShardManager>>,
	prev_id: Option<String>,
}

impl Components {
	pub fn new() -> Components {
		Components {
			config: None,
			mysql_client: None,
			cloned_shard_manager: None,
			prev_id: None,
		}
	}

	pub fn sets(
		&mut self,
		config: ConfigData,
		mysql_client: DatabaseConnection,
		cloned_shard_manager: Arc<ShardManager>,
	) {
		self.config = Some(config);
		self.mysql_client = Some(mysql_client);
		self.cloned_shard_manager = Some(cloned_shard_manager);
	}

	pub fn get_config(&self) -> &ConfigData {
		self.config.as_ref().unwrap()
	}

	pub fn get_prev_id(&self) -> Option<&String> {
		self.prev_id.as_ref()
	}

	pub fn set_prev_id(&mut self, val: String) {
		self.prev_id = Some(val);
	}

	pub fn get_shard_manager(&self) -> &Arc<ShardManager> {
		self.cloned_shard_manager.as_ref().unwrap()
	}

	pub fn get_sql_client(&self) -> &DatabaseConnection {
		self.mysql_client.as_ref().unwrap()
	}
}
