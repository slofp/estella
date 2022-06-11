use std::sync::Arc;
use log::{debug, error, info};
use serenity::client::{Cache, Context};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::id::{CommandId, GuildId};
use sqlx::Row;
use crate::{commands, exit, STATIC_COMPONENTS};
use crate::tables::account;
use crate::tables::quaryfn::{init_guild_config, insert_main_account_manual, insert_sub_account};

/*#[derive(sqlx::FromRow)]
struct UserData {
	uid: u64
}*/

pub async fn execute(ctx: Context, message: Message) {
	message_log(&message, ctx.cache.clone()).await;

	let lsc = STATIC_COMPONENTS.lock().await;
	let config = lsc.get_config();
	if *message.author.id.as_u64() != *config.get_owner_id() /*owner userid*/ {
		return;
	}
	std::mem::drop(lsc);

	if message.content == "estella.logout" {
		info!("logging out...");
		exit(true).await;
	}
	else if message.content.starts_with("estella.test") {
		test().await;
	}
	else if message.content.starts_with("estella.rep") {
		ping(&ctx, message).await;
	}
	else if message.content.starts_with("estella.create") {
		create(message, &ctx.http).await;
	}
	else if message.content.starts_with("estella.delete") {
		// estella.delete (command_id)
		delete(message, &ctx.http).await;
	}
	else if message.content.starts_with("estella.g_init") {
		guild_init(*message.guild_id.unwrap().as_u64()).await;
	}
	else if message.content.starts_with("estella.insert") {
		// estella.insert (uid) (name) (version) (is_sc) (is_leave)
		insert(&ctx, message).await;
	}
	else if message.content.starts_with("estella.insert_sub") {
		// estella.insert_sub (uid) (name) (main_uid)
		insert_sub(&ctx, message).await;
	}
}

async fn message_log(message: &Message, cache: Arc<Cache>) {
	info!("user: {}#{:04}\nChannel: {}\nkind: {:?}\ntext: {}\ncreate date: {}",
		message.author.name,
		message.author.discriminator,
		if message.is_private() { "DM".to_string() } else { message.channel_id.name(cache).await.unwrap_or_else(|| "None".to_string()) },
		message.kind,
		message.content,
		message.timestamp);
}

async fn insert_sub(ctx: &Context, message: Message) {
	let message_rep = message.content.replace("estella.insert_sub", "");
	let message_split = message_rep.trim().split(' ');
	let message_vec: Vec<&str> = message_split.collect::<Vec<&str>>();

	if let Err(error) = message.channel_id.send_message(
		&ctx.http,
		|m| m.content(format!("inserting: {}", message_vec[0]))
	).await {
		error!("{}", error);
	}

	let user_id: u64 = message_vec[0].parse().expect("Coundnt parse uid");
	let user_data = message.guild_id.unwrap().member(&ctx.http, user_id).await.unwrap();

	let insert_data = account::Sub {
		uid: user_id,
		name: message_vec[1].to_string(),
		guild_id: *message.guild_id.unwrap().as_u64(),
		join_date: user_data.joined_at.unwrap_or(message.guild_id.unwrap().created_at()),
		main_uid: message_vec[2].parse().expect("Coundnt parse main_uid"),
		first_cert: *message.author.id.as_u64(),
		second_cert: None
	};

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql();
	if let Err(error) = insert_sub_account(&insert_data, mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn insert(ctx: &Context, message: Message) {
	let message_rep = message.content.replace("estella.insert", "");
	let message_split = message_rep.trim().split(' ');
	let message_vec: Vec<&str> = message_split.collect::<Vec<&str>>();

	if let Err(error) = message.channel_id.send_message(
		&ctx.http,
		|m| m.content(format!("inserting: {}", message_vec[0]))
	).await {
		error!("{}", error);
	}

	let user_id: u64 = message_vec[0].parse().expect("Coundnt parse uid");
	let user_data = message.guild_id.unwrap().member(&ctx.http, user_id).await.unwrap();

	let insert_data = account::Main {
		uid: user_id,
		name: message_vec[1].to_string(),
		guild_id: *message.guild_id.unwrap().as_u64(),
		version: message_vec[2].parse().expect("Coundnt parse version"),
		join_date: user_data.joined_at.unwrap_or(message.guild_id.unwrap().created_at()),
		is_sc: message_vec[3].parse().expect("Coundnt parse is_sc"),
		is_leaved: message_vec[4].parse().expect("Coundnt parse is_leaved")
	};

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql();
	if let Err(error) = insert_main_account_manual(&insert_data, mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn guild_init(guild_id: u64) {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql();
	if let Err(error) = init_guild_config(guild_id, mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

async fn test() {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql();

	let res = sqlx::query("SELECT COUNT(*) as cnt FROM userdata")
		.fetch_one(mysql_client).await;

	std::mem::drop(lsc);

	if let Ok(res) = res {
		let cnt: i32 = res.get("cnt");
		info!("get value: {}", cnt);
		/*for row in res {
			println!("get value: {}", row.uid);
		}*/
	}
	else if let Err(res) = res {
		info!("{:?}", res);
	}
}

async fn ping(ctx: &Context, message: Message) {
	match message.reply_ping(
		ctx,
		"Wait pls..."
	).await {
		Ok(mut rep_message) => {
			let rep_time = rep_message.timestamp.timestamp_subsec_millis();
			let ping = rep_time - message.timestamp.timestamp_subsec_millis();

			if let Err(error) = rep_message.edit(
				ctx,
				move |m| m.content(
					format!("{}ms", ping)
				)
			).await {
				error!("{}", error);
			}
		},
		Err(error) => {
			error!("{}", error);
		}
	}
}

async fn create(message: Message, http: &Arc<Http>) {
	match create_command(message.guild_id.unwrap(), http).await {
		Ok(command) => {
			if let Err(error) = message.channel_id.send_message(
				http,
				|m| m.content(format!("success command create!\ncommand id: {}", command))
			).await {
				error!("{}", error);
			}
		},
		Err(error) => {
			if let Err(error2) = message.channel_id.send_message(
				http,
				|m| m.content(format!("create error: {}", error))
			).await {
				error!("{}", error);
				error!("{}", error2);
			}
		}
	};
}

async fn create_command(guild: GuildId, http: &Arc<Http>) -> Result<CommandId, serenity::Error> {
	let app_commands =
		guild.set_application_commands(http, commands::app_commands_build).await;

	debug!("adding response: {:#?}", app_commands);
	return match app_commands {
		Ok(app_commands) => {
			info!("success add command!");
			for command in &app_commands {
				info!("app id: {}", command.application_id);
				info!("command id: {}", command.id);
			}
			Ok(app_commands[0].id)
		},
		Err(error) => Err(error)
	}
}

async fn delete(message: Message, http: &Arc<Http>) {
	let rep_message = message.content.replace("estella.delete", "").replace(" ", "");
	if let Err(error) = message.channel_id.send_message(
		http,
		|m| m.content(format!("deleting: {}", rep_message))
	).await {
		error!("{}", error);
	}

	match message.guild_id.unwrap().delete_application_command(
		http,
		CommandId(
			rep_message
				.parse()
				.expect("Coundnt parse id")
		)
	).await {
		Ok(_) => {
			if let Err(error) = message.channel_id.send_message(
				http,
				|m| m.content("success command delete!")
			).await {
				error!("{}", error);
			}
		},
		Err(error) => {
			if let Err(error2) = message.channel_id.send_message(
				http,
				|m| m.content(format!("delete error: {}", error))
			).await {
				error!("{}", error);
				error!("{}", error2);
			}
		}
	};
}
