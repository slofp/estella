use log::{error, info};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryTrait};
use serenity::client::Context;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use crate::events::ready_event::DEL_PENDING_USERS;
use crate::STATIC_COMPONENTS;
use crate::utils::color;
use entity::enums::AccountType;

pub async fn execute(ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
	if guild_id.is_none() {
		return;
	}
	let guild_id = guild_id.unwrap();

	info!("Message Removed\nchannel_id: {}\nmessage_id: {}\nguild_id: {}", channel_id.as_u64(), deleted_message_id.as_u64(), guild_id.as_u64());

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	let pending_account =
		entity::PendingAccountBehavior::find()
			.filter(entity::pending_account::Column::GuildId.eq(guild_id.as_u64()))
			.filter(entity::pending_account::Column::MessageId.eq(deleted_message_id.as_u64()))
			.one(mysql_client)
			.await;
	std::mem::drop(lsc);

	if let Err(error) = pending_account {
		error!("DB Error: {:?}", error);
		return;
	}
	else if let Ok(None) = pending_account {
		return;
	}
	let pending_account = pending_account.unwrap().unwrap();

	if matches!(pending_account.account_type, AccountType::Main) {
		let mut lpu = DEL_PENDING_USERS.lock().await;
		lpu.push(pending_account.uid);
		std::mem::drop(lpu);
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();
	if let Err(error) = pending_account.clone().delete(mysql_client).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);

	if let Err(error) = channel_id.send_message(&ctx.http, |cm| {
		cm
			.add_embed(|ce| {
				ce
					.title("取り消されました")
					.description("申請用メッセージが削除されたため以下の申請を自動的に取り消しました。")
					.field("ユーザーID", pending_account.uid, true)
					.field("名前", pending_account.name, true)
					.field("アカウントタイプ", pending_account.account_type.to_string(), true)
					.color(color::warning_color())
			})
	}).await {
		error!("Error: {}", error);
	}
}
