use log::{error, info, warn};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::user::User;
use entity::{GuildConfig, GuildConfigBehavior, MainAccountBehavior, SubAccount, SubAccountBehavior};
use crate::STATIC_COMPONENTS;
use crate::utils::{color, convert};
use crate::utils::convert::format_discord_username;

pub async fn execute(ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
	info!("member removed");
	info!("  username: {}", format_discord_username(&user));

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let guild_config =
			GuildConfigBehavior::find_by_id(guild_id.as_u64())
				.one(mysql_client)
				.await;
	std::mem::drop(lsc);
	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	else if let Ok(None) = guild_config {
		error!("Not found GuildConfig.");
		return;
	}
	let guild_config = guild_config.unwrap().unwrap();

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
	let mysql_client = lsc.get_sql_client();
	let mem_account =
		SubAccountBehavior::find_by_id(user.id.as_u64())
			.filter(entity::sub_account::Column::GuildId.eq(guild_id.as_u64()))
			.one(mysql_client)
			.await;
	if let Ok(Some(mem_account)) = mem_account {
		is_sub = true;
		if let Err(error) = mem_account.delete(mysql_client).await {
			error!("DB Error: {:?}", error);
		}
	}
	else if let Ok(None) = mem_account {
		warn!("Not found member account from sub.");
	}
	else if let Err(error) = mem_account {
		error!("DB Error: {:?}", error);
	}

	let mem_accounts =
		MainAccountBehavior::find_by_id(user.id.as_u64())
			.find_with_related(entity::sub_account::Entity)
			.filter(guild_id.as_u64())
			.all(mysql_client)
			.await;
	if let Ok(mut mem_accounts) = mem_accounts {
		let (mem_account, main_sub_accounts) = mem_accounts[0].to_owned();
		for main_sub_account in main_sub_accounts {
			if let Err(error) = guild_id.kick(&ctx.http, main_sub_account.uid).await {
				error!("{}", error);
			}
		}

		let mut mem_account = mem_account.into_active_model();
		mem_account.is_leaved = Set(true);
		if let Err(error) = mem_account.update(mysql_client).await {
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
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("Botをサーバーから削除しました")
				.description("以下のBotを削除しました。")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::normal_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_remove_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("サーバーから削除しました")
				.description("以下のユーザーを削除しました。またサーバーに入れる場合は再度申請をしてください")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::failed_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_ban_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("サーバーを抜けました")
				.description("以下のユーザーはサーバーを抜けたためBANされました")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::critical_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}
