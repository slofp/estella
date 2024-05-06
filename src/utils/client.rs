use std::sync::Arc;
use sea_orm::DatabaseConnection;
use serenity::all::ShardManager;
use crate::ConfigData;

pub struct Components {
	config: Option<ConfigData>,
	mysql_client: Option<DatabaseConnection>,
	cloned_shard_manager: Option<Arc<ShardManager>>
}

impl Components {
	pub fn new() -> Components {
		Components {
			config: None,
			mysql_client: None,
			cloned_shard_manager: None
		}
	}

	pub fn sets(&mut self, config: ConfigData, mysql_client: DatabaseConnection, cloned_shard_manager: Arc<ShardManager>) {
		self.config = Some(config);
		self.mysql_client = Some(mysql_client);
		self.cloned_shard_manager = Some(cloned_shard_manager);
	}

	pub fn get_config(&self) -> &ConfigData {
		self.config.as_ref().unwrap()
	}

	pub fn get_shard_manager(&self) -> &Arc<ShardManager> {
		self.cloned_shard_manager.as_ref().unwrap()
	}

	pub fn get_sql_client(&self) -> &DatabaseConnection {
		self.mysql_client.as_ref().unwrap()
	}
}
