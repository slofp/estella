use sqlx::{Error, MySql, Pool};
use crate::tables::account;

pub async fn get_main_account(uid: u64, client: &Pool<MySql>) -> Result<account::Main, Error> {
	sqlx::query_as::<_, account::Main>("select * from main_account where uid = ?")
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_sub_account(uid: u64, client: &Pool<MySql>) -> Result<account::Sub, Error> {
	sqlx::query_as::<_, account::Sub>("select * from sub_account where uid = ?")
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_pending_account(uid: u64, client: &Pool<MySql>) -> Result<account::Pending, Error> {
	sqlx::query_as::<_, account::Pending>("select * from pending_account where uid = ?")
		.bind(uid)
		.fetch_one(client).await
}
