use log::{error, info, warn};
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::user::User;
use crate::STATIC_COMPONENTS;
use crate::tables::quaryfn::{delete_main_account, delete_sub_account, get_guild_config, get_main_account, get_main_sub_account, get_sub_account, update_main_account};
use crate::utils::{color, convert};

pub async fn execute(ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
	info!("member removed");
	info!("  username: {}#{:04}", user.name, user.discriminator);

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let guild_config = get_guild_config(*guild_id.as_u64(), locked_db).await;
	std::mem::drop(lsc);
	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	let guild_config = guild_config.unwrap();

	if !guild_config.leave_ban {
		info!("this guild not enabled leave ban");
		return;
	}

	if user.bot {
		info!("user is bot");
		if let Some(log_channel_id) = guild_config.log_channel_id {
			send_bot_message(&ctx, log_channel_id, &user).await;
		}
		else {
			warn!("log channel is not found");
		}
		return;
	}

	let mut is_sub = false;
	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let mem_account = get_sub_account(*guild_id.as_u64(), *user.id.as_u64(), &locked_db).await;
	if let Ok(mem_account) = mem_account {
		is_sub = true;
		if let Err(error) = delete_sub_account(&mem_account, &locked_db).await {
			error!("DB Error: {:?}", error);
		}
	}
	else if let Err(error) = mem_account {
		error!("DB Error: {:?}", error);
	}

	let mem_account = get_main_account(*guild_id.as_u64(), *user.id.as_u64(), &locked_db).await;
	if let Ok(mut mem_account) = mem_account {
		let main_sub_accounts = get_main_sub_account(mem_account.uid, &locked_db).await;
		if let Ok(main_sub_accounts) = main_sub_accounts {
			for main_sub_account in main_sub_accounts {
				if let Err(error) = guild_id.kick(&ctx.http, main_sub_account.uid).await {
					error!("{}", error);
				}
			}
		}

		mem_account.is_leaved = true;
		if let Err(error) = update_main_account(&mem_account, &locked_db).await {
			error!("DB Error: {:?}", error);
		}
	}
	std::mem::drop(lsc);

	if is_sub {
		if let Some(log_channel_id) = guild_config.log_channel_id {
			send_remove_message(&ctx, log_channel_id, &user).await;
		}
		else {
			warn!("log channel is not found");
		}
	}
	else {
		if let Some(log_channel_id) = guild_config.log_channel_id {
			send_ban_message(&ctx, log_channel_id, &user).await;
		}
		else {
			warn!("log channel is not found");
		}

		if let Some(member) = member_data_if_available {
			info!("member data found!");
			if let Err(error) = member.ban(&ctx.http, 0).await {
				error!("{}", error);
			}
		}
	}
}

async fn send_bot_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("Botをサーバーから削除しました")
				.description("以下のBotを削除しました。")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::username(usr.name.clone(), usr.discriminator), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::normal_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_remove_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("サーバーから削除しました")
				.description("以下のユーザーを削除しました。またサーバーに入れる場合は再度申請をしてください")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::username(usr.name.clone(), usr.discriminator), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::failed_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_ban_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("サーバーを抜けました")
				.description("以下のユーザーはサーバーを抜けたためBANされました")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::username(usr.name.clone(), usr.discriminator), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::critical_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}
