use log::{error, info};
use serenity::client::Context;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use crate::events::ready_event::DEL_PENDING_USERS;
use crate::STATIC_COMPONENTS;
use crate::tables::quaryfn::{delete_pending_account, get_pending_account_from_message_id};
use crate::utils::color;
use crate::utils::enums::AccountType;

pub async fn execute(ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
	if guild_id.is_none() {
		return;
	}
	let guild_id = guild_id.unwrap();

	info!("Message Removed\nchannel_id: {}\nmessage_id: {}\nguild_id: {}", channel_id.as_u64(), deleted_message_id.as_u64(), guild_id.as_u64());

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let pending_account = get_pending_account_from_message_id(*guild_id.as_u64(), *deleted_message_id.as_u64(), locked_db).await;
	std::mem::drop(lsc);

	if let Err(error) = pending_account {
		error!("DB Error: {:?}", error);
		return;
	}
	let pending_account = pending_account.unwrap();

	if matches!(pending_account.account_type, AccountType::Main) {
		let mut lpu = DEL_PENDING_USERS.lock().await;
		lpu.push(pending_account.uid);
		std::mem::drop(lpu);
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	if let Err(error) = delete_pending_account(&pending_account, &locked_db).await {
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
