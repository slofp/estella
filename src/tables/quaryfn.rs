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

pub async fn get_confirmed_account(uid: u64, client: &Pool<MySql>) -> Result<account::Confirmed, Error> {
	sqlx::query_as::<_, account::Confirmed>("select * from confirmed_account where uid = ?")
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
	sqlx::query("insert into main_account values (?, ?, ?, ?, default, default)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.version)
		.bind(&value.join_date)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into sub_account values (?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.join_date)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.bind(&value.second_cert)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_confirmed_account(value: &account::Confirmed, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into confirmed_account values (?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.account_type)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into pending_account values (?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.end_voting)
		.execute(client).await?;

	Ok(())
}

/// update func

pub async fn update_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update main_account set is_leaved = ? where uid = ?")
		.bind(&value.is_leaved)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

// 今の所使用しない
/*pub async fn update_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update sub_account set ")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.join_date)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.bind(&value.second_cert)
		.fetch_one(client).await?;

	Ok(())
}*/

// 今の所使用しない
/*pub async fn update_confirmed_account(value: &account::Confirmed, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update confirmed_account set ")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.account_type)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.fetch_one(client).await?;

	Ok(())
}*/

// 今の所使用しない
/*pub async fn update_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update pending_account set ")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.end_voting)
		.fetch_one(client).await?;

	Ok(())
}*/

/// delete func

pub async fn delete_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from main_account where uid = ?")
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn delete_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from sub_account where uid = ?")
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn delete_confirmed_account(value: &account::Confirmed, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from confirmed_account where uid = ?")
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn delete_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from pending_account where uid = ?")
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

/// upsert func

/// bot admin control func

pub async fn insert_main_account_manual(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into main_account values (?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.version)
		.bind(&value.join_date)
		.bind(&value.is_sc)
		.bind(&value.is_leaved)
		.execute(client).await?;

	Ok(())
}
