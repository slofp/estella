use log::error;
use sqlx::{Error, MySql, Pool};
use crate::tables::{account, guild};

/// select func

pub async fn get_main_account(guild_id: u64, uid: u64, client: &Pool<MySql>) -> Result<account::Main, Error> {
	sqlx::query_as::<_, account::Main>("select * from main_account where guild_id = ? and uid = ?")
		.bind(guild_id)
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_sub_account(guild_id: u64, uid: u64, client: &Pool<MySql>) -> Result<account::Sub, Error> {
	sqlx::query_as::<_, account::Sub>("select * from sub_account where guild_id = ? and uid = ?")
		.bind(guild_id)
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_confirmed_account(guild_id: u64, uid: u64, client: &Pool<MySql>) -> Result<account::Confirmed, Error> {
	sqlx::query_as::<_, account::Confirmed>("select * from confirmed_account where guild_id = ? and uid = ?")
		.bind(guild_id)
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_pending_account(guild_id: u64, uid: u64, client: &Pool<MySql>) -> Result<account::Pending, Error> {
	sqlx::query_as::<_, account::Pending>("select * from pending_account where guild_id = ? and uid = ?")
		.bind(guild_id)
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_pending_account_from_message_id(guild_id: u64, message_id: u64, client: &Pool<MySql>) -> Result<account::Pending, Error> {
	sqlx::query_as::<_, account::Pending>("select * from pending_account where guild_id = ? and message_id = ?")
		.bind(guild_id)
		.bind(message_id)
		.fetch_one(client).await
}

pub async fn get_all_main_pending_account(client: &Pool<MySql>) -> Result<Vec<account::Pending>, Error> {
	sqlx::query_as::<_, account::Pending>("select * from pending_account where account_type = 1")
		.fetch_all(client).await
}

pub async fn get_guild_config(uid: u64, client: &Pool<MySql>) -> Result<guild::Config, Error> {
	sqlx::query_as::<_, guild::Config>("select * from guild_config where uid = ?")
		.bind(uid)
		.fetch_one(client).await
}

pub async fn get_main_sub_account(main_id: u64, client: &Pool<MySql>) -> Result<Vec<account::Sub>, Error> {
	sqlx::query_as::<_, account::Sub>("select * from sub_account where main_uid = ?")
		.bind(main_id)
		.fetch_all(client).await
}

pub async fn exist_user_id(guild_id: u64, user_id: u64, client: &Pool<MySql>) -> bool {
	let join_task = tokio::join!(
		get_main_account(guild_id, user_id, &client),
		get_sub_account(guild_id, user_id, &client),
		get_confirmed_account(guild_id, user_id, &client),
		get_pending_account(guild_id, user_id, &client)
	);

	if let Err(error) = join_task.0 {
		error!("DB Error: {:?}", error);
		if let Err(error) = join_task.1 {
			error!("DB Error: {:?}", error);
			if let Err(error) = join_task.2 {
				error!("DB Error: {:?}", error);
				if let Err(error) = join_task.3 {
					error!("DB Error: {:?}", error);
					return false;
				}
			}
		}
	}

	return true;
}

/// insert func

pub async fn insert_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into main_account values (?, ?, ?, ?, ?, default, default)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.guild_id)
		.bind(&value.version)
		.bind(&value.join_date)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into sub_account values (?, ?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.guild_id)
		.bind(&value.join_date)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.bind(&value.second_cert)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_confirmed_account(value: &account::Confirmed, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into confirmed_account values (?, ?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.guild_id)
		.bind(&value.account_type)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.bind(&value.second_cert)
		.execute(client).await?;

	Ok(())
}

pub async fn insert_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into pending_account values (?, ?, ?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.guild_id)
		.bind(&value.account_type)
		.bind(&value.message_id)
		.bind(&value.end_voting)
		.bind(&value.main_uid)
		.bind(&value.first_cert)
		.execute(client).await?;

	Ok(())
}

pub async fn init_user_data(uid: u64, glacialeur: Option<String>, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into user_data(uid, glacialeur) values (?, ?)")
		.bind(uid)
		.bind(glacialeur)
		.execute(client).await?;

	Ok(())
}

pub async fn init_guild_config(uid: u64, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into guild_config(uid) values (?)")
		.bind(uid)
		.execute(client).await?;

	Ok(())
}

/// update func

pub async fn update_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update main_account set is_leaved = ? where guild_id = ? and uid = ?")
		.bind(&value.is_leaved)
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn update_guild_config_log(guild_id: u64, id: u64, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update guild_config set log_channel_id = ? where uid = ?")
		.bind(id)
		.bind(guild_id)
		.execute(client).await?;

	Ok(())
}

pub async fn update_guild_config_auth(guild_id: u64, id: u64, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update guild_config set auth_role_id = ? where uid = ?")
		.bind(id)
		.bind(guild_id)
		.execute(client).await?;

	Ok(())
}

pub async fn update_guild_config_bot(guild_id: u64, id: u64, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update guild_config set bot_role_id = ? where uid = ?")
		.bind(id)
		.bind(guild_id)
		.execute(client).await?;

	Ok(())
}

pub async fn update_guild_config_white(guild_id: u64, value: bool, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update guild_config set white_list = ? where uid = ?")
		.bind(value)
		.bind(guild_id)
		.execute(client).await?;

	Ok(())
}

pub async fn update_guild_config_leave(guild_id: u64, value: bool, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update guild_config set leave_ban = ? where uid = ?")
		.bind(value)
		.bind(guild_id)
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

pub async fn update_pending_sub_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("update pending_account set first_cert = ? where guild_id = ? and uid = ?")
		.bind(&value.first_cert)
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

/// delete func

/*pub async fn delete_main_account(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from main_account where guild_id = ? and uid = ?")
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}*/

pub async fn delete_sub_account(value: &account::Sub, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from sub_account where guild_id = ? and uid = ?")
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn delete_confirmed_account(value: &account::Confirmed, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from confirmed_account where guild_id = ? and uid = ?")
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

pub async fn delete_pending_account(value: &account::Pending, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("delete from pending_account where guild_id = ? and uid = ?")
		.bind(&value.guild_id)
		.bind(&value.uid)
		.execute(client).await?;

	Ok(())
}

/// upsert func

/// bot admin control func

pub async fn insert_main_account_manual(value: &account::Main, client: &Pool<MySql>) -> Result<(), Error> {
	sqlx::query("insert into main_account values (?, ?, ?, ?, ?, ?, ?)")
		.bind(&value.uid)
		.bind(&value.name)
		.bind(&value.guild_id)
		.bind(&value.version)
		.bind(&value.join_date)
		.bind(&value.is_sc)
		.bind(&value.is_leaved)
		.execute(client).await?;

	Ok(())
}
