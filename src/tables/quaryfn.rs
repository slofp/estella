use sqlx::{Error, MySql, Pool};
use crate::tables::account;

/// select func

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

/// insert func

pub async fn insert_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into main_account values ($1, $2, $3, $4, default, default)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.version)
		.bind(&value.join_date)
		.fetch_one(client).await?;

	Ok(())
}

pub async fn insert_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into sub_account values ($1, $2, $3, $4, $5, $6)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.join_date)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.bind(&value.second_cert)
		.fetch_one(client).await?;

	Ok(())
}

pub async fn insert_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into pending_account values ($1, $2, $3)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.end_voting)
		.fetch_one(client).await?;

	Ok(())
}

/// update func

/// delete func

/// bot admin control func
