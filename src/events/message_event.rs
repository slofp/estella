use crate::chat::{create_user_message, getchat_responce};
use crate::utils::convert::format_discord_username;
use crate::utils::glacialeur;
use crate::{commands, exit, STATIC_COMPONENTS};
use log::{debug, error, info};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, PaginatorTrait};
use serenity::all::{CacheHttp, CreateMessage, EditMessage};
use serenity::client::Context;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::id::{CommandId, GuildId};
use std::sync::Arc;

/*#[derive(sqlx::FromRow)]
struct UserData {
	uid: u64
}*/

pub async fn execute(ctx: Context, message: Message) {
	message_log(&message, &ctx).await;

	let lsc = STATIC_COMPONENTS.lock().await;
	let config = lsc.get_config();
	if message.author.id.get() != *config.get_owner_id()
	/*owner userid*/
	{
		return;
	}
	std::mem::drop(lsc);

	if message.content == "estella.logout" {
		info!("logging out...");
		exit(true).await;
	} else if message.content.starts_with("estella.test") {
		test().await;
	} else if message.content.starts_with("estella.rep") {
		ping(&ctx, message).await;
	} else if message.content.starts_with("estella.create") {
		create(message, &ctx.http).await;
	} else if message.content.starts_with("estella.delete") {
		// estella.delete (command_id)
		delete(message, &ctx.http).await;
	} else if message.content.starts_with("estella.g_init") {
		guild_init(message.guild_id.unwrap().get()).await;
	} else if message.content.starts_with("estella.insert") {
		// estella.insert (uid) (name) (version) (is_sc) (is_leave)
		insert(&ctx, message).await;
	} else if message.content.starts_with("estella.sub_insert") {
		// estella.sub_insert (uid) (name) (main_uid)
		insert_sub(&ctx, message).await;
	} else if message.content.starts_with("estella.chat_test") {
		// estella.chat_test (msg)
		test_chat(&ctx, message).await;
	}
}

async fn test_chat(ctx: &Context, message: Message) {
	let message_rep = message.content.replace("estella.chat_test", "");
	let message_split = message_rep.trim().split(' ');
	let message_vec: Vec<&str> = message_split.collect::<Vec<&str>>();

	let comp_lock = STATIC_COMPONENTS.lock().await;
	let prev_id = comp_lock.get_prev_id().map(|v| v.clone());
	std::mem::drop(comp_lock);

	let (data, id) = getchat_responce(
		create_user_message(
			message_vec[3..].join(" "),
			message_vec[0].parse().expect("could not parse level"),
			message_vec[1],
			if message_vec[2] == "M" { entity::enums::Gender::Male } else { entity::enums::Gender::Ladies },
			&chrono::Local::now()
		),
		prev_id
	).await.unwrap();

	let mut comp_lock = STATIC_COMPONENTS.lock().await;
	comp_lock.set_prev_id(id);
	std::mem::drop(comp_lock);

	message.channel_id
		.send_message(ctx, CreateMessage::new().content(data.message)).await.unwrap();
}

async fn message_log(message: &Message, cache: impl CacheHttp) {
	info!(
		"user: {}\nChannel: {}\nkind: {:?}\ntext: {}\ncreate date: {}",
		format_discord_username(&message.author),
		if message.is_private() {
			"DM".to_string()
		} else {
			message
				.channel_id
				.name(cache)
				.await
				.unwrap_or_else(|_| "None".to_string())
		},
		message.kind,
		message.content,
		message.timestamp
	);
}

async fn insert_sub(ctx: &Context, message: Message) {
	let message_rep = message.content.replace("estella.sub_insert", "");
	let message_split = message_rep.trim().split(' ');
	let message_vec: Vec<&str> = message_split.collect::<Vec<&str>>();

	if let Err(error) = message
		.channel_id
		.send_message(
			&ctx.http,
			CreateMessage::new().content(format!("inserting: {}", message_vec[0])),
		)
		.await
	{
		error!("{}", error);
	}

	let user_id: u64 = message_vec[0].parse().expect("Coundnt parse uid");
	let user_data = message.guild_id.unwrap().member(&ctx.http, user_id).await.unwrap();

	let insert_data = entity::SubAccount {
		uid: user_id,
		name: message_vec[1].to_string(),
		guild_id: message.guild_id.unwrap().get(),
		join_date: user_data
			.joined_at
			.unwrap_or(message.guild_id.unwrap().created_at())
			.to_utc(),
		main_uid: message_vec[2].parse().expect("Coundnt parse main_uid"),
		first_cert: message.author.id.get(),
		second_cert: None,
	};

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let insert_res = insert_data.into_active_model().insert(mysql_client).await;
	if let Err(error) = insert_res {
		error!("DB Error: {:?}", error);
		return;
	}
	let insert_data = insert_res.unwrap();

	let user_data = entity::UserData {
		uid: insert_data.uid,
		glacialeur: None,
		call_name: None,
		gender: None,
		likability_level: None,
	};
	if let Err(error) = user_data.into_active_model().insert(mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn insert(ctx: &Context, message: Message) {
	let message_rep = message.content.replace("estella.insert", "");
	let message_split = message_rep.trim().split(' ');
	let message_vec: Vec<&str> = message_split.collect::<Vec<&str>>();

	if let Err(error) = message
		.channel_id
		.send_message(
			&ctx.http,
			CreateMessage::new().content(format!("inserting: {}", message_vec[0])),
		)
		.await
	{
		error!("{}", error);
	}

	let user_id: u64 = message_vec[0].parse().expect("Coundnt parse uid");
	let user_data = message.guild_id.unwrap().member(&ctx.http, user_id).await.unwrap();

	let insert_data = entity::MainAccount {
		uid: user_id,
		name: message_vec[1].to_string(),
		guild_id: message.guild_id.unwrap().get(),
		version: message_vec[2].parse().expect("Coundnt parse version"),
		join_date: user_data
			.joined_at
			.unwrap_or(message.guild_id.unwrap().created_at())
			.to_utc(),
		is_server_creator: message_vec[3].parse().expect("Coundnt parse is_server_creator"),
		is_leaved: message_vec[4].parse().expect("Coundnt parse is_leaved"),
	};
	let g_str = glacialeur::generate(
		insert_data.uid,
		insert_data.version,
		insert_data.join_date.timestamp() - message.guild_id.unwrap().created_at().timestamp(),
	);

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let insert_res = insert_data.into_active_model().insert(mysql_client).await;
	if let Err(error) = insert_res {
		error!("DB Error: {:?}", error);
		return;
	}
	let insert_data = insert_res.unwrap();

	let user_data = entity::UserData {
		uid: insert_data.uid,
		glacialeur: Some(g_str),
		call_name: None,
		gender: None,
		likability_level: None,
	};
	if let Err(error) = user_data.into_active_model().insert(mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn guild_init(guild_id: u64) {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();

	let guild_config = entity::GuildConfig {
		uid: guild_id,
		white_list: false,
		leave_ban: false,
		log_channel_id: None,
		auth_role_id: None,
		bot_role_id: None,
		send_ai_chat_channel_id: None,
	};
	if let Err(error) = guild_config.into_active_model().insert(mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn test() {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();

	let res = entity::UserDataBehavior::find().count(mysql_client).await;

	std::mem::drop(lsc);

	if let Ok(cnt) = res {
		info!("get value: {}", cnt);
		/*for row in res {
			println!("get value: {}", row.uid);
		}*/
	} else if let Err(res) = res {
		info!("{:?}", res);
	}
}

async fn ping(ctx: &Context, message: Message) {
	match message.reply_ping(ctx, "Wait pls...").await {
		Ok(mut rep_message) => {
			let rep_time = rep_message.timestamp.timestamp_subsec_millis();
			let ping = rep_time - message.timestamp.timestamp_subsec_millis();

			if let Err(error) = rep_message
				.edit(ctx, EditMessage::new().content(format!("{}ms", ping)))
				.await
			{
				error!("{}", error);
			}
		},
		Err(error) => {
			error!("{}", error);
		},
	}
}

async fn create(message: Message, http: &Arc<Http>) {
	match create_command(message.guild_id.unwrap(), http).await {
		Ok(command) => {
			if let Err(error) = message
				.channel_id
				.send_message(
					http,
					CreateMessage::new().content(format!("success command create!\ncommand id: {}", command)),
				)
				.await
			{
				error!("{}", error);
			}
		},
		Err(error) => {
			if let Err(error2) = message
				.channel_id
				.send_message(http, CreateMessage::new().content(format!("create error: {}", error)))
				.await
			{
				error!("{}", error);
				error!("{}", error2);
			}
		},
	};
}

async fn create_command(guild: GuildId, http: &Arc<Http>) -> Result<CommandId, serenity::Error> {
	let command = guild.create_command(http, commands::app_commands_build()).await;

	debug!("adding response: {:#?}", command);
	return match command {
		Ok(command) => {
			info!("success add command!");
			info!("app id: {}", command.application_id);
			info!("command id: {}", command.id);
			Ok(command.id)
		},
		Err(error) => Err(error),
	};
}

async fn delete(message: Message, http: &Arc<Http>) {
	let rep_message = message.content.replace("estella.delete", "").replace(" ", "");
	if let Err(error) = message
		.channel_id
		.send_message(http, CreateMessage::new().content(format!("deleting: {}", rep_message)))
		.await
	{
		error!("{}", error);
	}

	match message
		.guild_id
		.unwrap()
		.delete_command(http, CommandId::new(rep_message.parse().expect("Coundnt parse id")))
		.await
	{
		Ok(_) => {
			if let Err(error) = message
				.channel_id
				.send_message(http, CreateMessage::new().content("success command delete!"))
				.await
			{
				error!("{}", error);
			}
		},
		Err(error) => {
			if let Err(error2) = message
				.channel_id
				.send_message(http, CreateMessage::new().content(format!("delete error: {}", error)))
				.await
			{
				error!("{}", error);
				error!("{}", error2);
			}
		},
	};
}
