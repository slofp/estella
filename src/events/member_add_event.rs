use log::{error, info, warn};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter};
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::user::User;
use crate::STATIC_COMPONENTS;
use crate::utils::{color, convert, glacialeur};
use entity::enums::AccountType;
use entity::{ConfirmedAccountBehavior, GuildConfigBehavior, MainAccount, SubAccount, UserData};
use crate::utils::convert::{flatten_result_option, format_discord_username};

pub async fn execute(ctx: Context, guild_id: GuildId, mut new_member: Member) {
	info!("new member!");
	info!("  username: {}", format_discord_username(&new_member.user));

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
		error!("not found guild config.");
		return;
	}
	let guild_config = guild_config.unwrap().unwrap();

	if !guild_config.white_list {
		info!("this guild not enabled white list");
		return;
	}
	if new_member.user.bot {
		info!("is bot");
		if let Some(log_channel_id) = guild_config.log_channel_id {
			send_bot_message(&ctx, log_channel_id, &new_member.user).await;
		}
		else {
			warn!("log channel is not found");
		}
		if let Some(role_id) = guild_config.bot_role_id {
			if let Err(error) = new_member.add_role(&ctx.http, role_id).await {
				error!("Error: {:?}", error);
			}
		}
		else {
			warn!("bot_role_id is none");
		}
		return;
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let member_account =
		flatten_result_option(
			ConfirmedAccountBehavior::find_by_id(new_member.user.id.as_u64())
				.filter(entity::confirmed_account::Column::GuildId.eq(guild_config.uid))
				.one(mysql_client)
				.await
		);
	std::mem::drop(lsc);
	if let Err(error) = member_account {
		// member kick when if not exist account from db.
		error!("DB Error: {:?}", error);

		if let Some(log_channel_id) = guild_config.log_channel_id {
			send_kicked_message(&ctx, log_channel_id, &new_member.user).await;
		}
		else {
			warn!("log channel is not found");
		}

		if let Err(kick_error) = new_member.kick(&ctx.http).await {
			error!("Error: {:?}", kick_error);
		}
		return;
	}
	let member_account = member_account.unwrap();

	if let Some(log_channel_id) = guild_config.log_channel_id {
		send_success_message(&ctx, log_channel_id, &member_account.account_type, &new_member.user).await;
	}
	else {
		warn!("log channel is not found");
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	if let Err(error) = member_account.clone().delete(mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	let mut g_str: Option<String> = None;
	if matches!(member_account.account_type, AccountType::Main) {
		let main_account = MainAccount {
			uid: member_account.uid,
			name: member_account.name,
			guild_id: member_account.guild_id,
			version: 1 << 3,
			join_date: new_member.joined_at.unwrap_or_else(|| chrono::Utc::now()),
			is_server_creator: false,
			is_leaved: false
		};
		let insert_res =
				main_account
					.into_active_model()
					.insert(mysql_client)
					.await;
		if let Err(error) = insert_res {
			error!("DB Error: {:?}", error);
			return;
			// TODO: もしエラーだった場合のリカバリを考える
		}
		let main_account = insert_res.unwrap();
		g_str = Some(glacialeur::generate(
			main_account.uid,
			main_account.version,
			main_account.join_date.timestamp() - guild_id.created_at().timestamp()
		));
	}
	else {
		let sub_account = SubAccount {
			uid: member_account.uid,
			name: member_account.name,
			guild_id: member_account.guild_id,
			join_date: new_member.joined_at.unwrap_or_else(|| chrono::Utc::now()),
			main_uid: member_account.main_uid.unwrap(),
			first_cert: member_account.first_cert.unwrap(),
			second_cert: member_account.second_cert
		};
		if let Err(error) = sub_account.into_active_model().insert(mysql_client).await {
			error!("DB Error: {:?}", error);
		}
	}
	std::mem::drop(lsc);

	if let Some(role_id) = guild_config.auth_role_id {
		if let Err(error) = new_member.add_role(&ctx.http, role_id).await {
			error!("Error: {:?}", error);
		}
	}
	else {
		warn!("auth_role_id is none");
	}


	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let user_data = UserData {
		uid: member_account.uid,
		glacialeur: g_str
	};
	if let Err(error) = user_data.into_active_model().insert(mysql_client).await {
		error!("DB Error: {:?}", error);
	}

	std::mem::drop(lsc);
}

async fn send_bot_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("Botが追加されました")
				.description("以下のBotが追加されました。")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::normal_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_kicked_message(ctx: &Context, channel_id: u64, usr: &User) {
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("ブロックされました")
				.description("以下のユーザーは未承認なので入ることができません。")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::failed_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}

async fn send_success_message(ctx: &Context, channel_id: u64, a_type: &AccountType, usr: &User) {
	let log_channel = ChannelId::from(channel_id);
	if let Err(error) = log_channel.send_message(&ctx.http, |cm| {
		cm.add_embed(|e| {
			e
				.title("許可されました")
				.description("以下のユーザーは承認済みのため入鯖を許可しました。")
				.field("ID", usr.id, true)
				.field("ユーザー名", convert::format_discord_username(usr), true)
				.field("アカウントタイプ", a_type.to_string(), true)
				.thumbnail(usr.avatar_url().unwrap_or_else(|| "".to_string()))
				.color(color::success_color())
		})
	}).await {
		error!("Error: {:?}", error);
	}
}
