use std::sync::Arc;
use serenity::client::bridge::gateway::ShardManager;
use sqlx::{MySql, Pool};
use tokio::sync::Mutex;
use crate::ConfigData;

pub struct Components {
	config: Option<ConfigData>,
	mysql_client: Option<Pool<MySql>>,
	cloned_shardmanager: Option<Arc<Mutex<ShardManager>>>
}

impl Components {
	pub fn new() -> Components {
		Components {
			config: None,
			mysql_client: None,
			cloned_shardmanager: None
		}
	}

	pub fn sets(&mut self, config: ConfigData, mysql_client: Pool<MySql>, cloned_shardmanager: Arc<Mutex<ShardManager>>) {
		self.config = Some(config);
		self.mysql_client = Some(mysql_client);
		self.cloned_shardmanager = Some(cloned_shardmanager);
	}

	pub fn get_config(&self) -> &ConfigData {
		self.config.as_ref().unwrap()
	}

	pub fn get_sm(&self) -> &Arc<Mutex<ShardManager>> {
		self.cloned_shardmanager.as_ref().unwrap()
	}

	pub fn get_sql(&self) -> &Pool<MySql> {
		self.mysql_client.as_ref().unwrap()
	}
}
