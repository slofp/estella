use std::sync::Arc;
use log::{debug, error, info};
use serenity::client::{Cache, Context};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::id::{CommandId, GuildId};
use sqlx::Row;
use crate::{commands, STATIC_COMPONENTS};

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
	if message.content == "estella.logout" {
		info!("logging out...");
		exit().await;
	}
	else if message.content.starts_with("estella.test") {
		info!("estella.test execute!");
		test().await;
	}
	else if message.content.starts_with("estella.rep") {
		ping(&ctx, message).await;
	}
	else if message.content.starts_with("estella.create") {
		create(message, &ctx.http).await;
	}
	else if message.content.starts_with("estella.delete") {
		delete(message, &ctx.http).await;
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

async fn exit() {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mut locked_shardmanager = lsc.get_sm().lock().await;
	locked_shardmanager.shutdown_all().await;

	info!("Exiting...");

	while locked_shardmanager.shards_instantiated().await.len() != 0 { }
	info!("Bot logged out.");

	lsc.get_sql().close().await;
	info!("Database closed.");

	std::process::exit(0);
}

async fn test() {
	//let t = message.content.replace("estella.test", "").replace(" ", "");
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql();

	let res = sqlx::query("SELECT COUNT(*) as cnt FROM userdata")
		.fetch_one(mysql_client).await;

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
