use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, LazyLock}};

use entity::{GuildConfigBehavior, UserData, UserDataBehavior};
use sea_orm::EntityTrait;
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, Mentionable, Message, UserId};
use tokio::sync::Mutex;

use crate::{chat::{create_user_message_with_image_url, getchat_responce, param::calculate_likability_level_from_message_count}, STATIC_COMPONENTS};

pub(super) static GUILD_CHATTING_STATE: LazyLock<Mutex<HashMap<GuildId, AtomicBool>>> = LazyLock::new(|| {
	Mutex::new(HashMap::new())
});

pub(super) static GUILD_CHATTING_PREV_ID: LazyLock<Mutex<HashMap<GuildId, String>>> = LazyLock::new(|| {
	Mutex::new(HashMap::new())
});

pub(crate) async fn chat_for_message(ctx: &Context, message: Message) {
	let guild_id = message.guild(&ctx.cache).unwrap().id;
	let channel = message.channel_id;
	if !check_send_target_channel_id(guild_id, channel).await {
		return;
	}

	let mut lock = GUILD_CHATTING_STATE.lock().await;
	let guild_atomic = lock.get(&guild_id);
	let is_execute = guild_atomic.map(|v| v.load(Ordering::Acquire)).unwrap_or(false);
	if is_execute {
		return;
	}

	let user = &message.author;
	let user_data = get_target_user_data(user.id).await;
	if user_data.is_none() {
		message.reply(ctx, "ごめんね、あなたが誰なのかわからなかったからちゃんとした返事が出来なかったの。設定してもらってもいいかな？").await.unwrap();
		return;
	}
	let user_data = user_data.unwrap();

	if guild_atomic.is_none() {
		lock.insert(guild_id, AtomicBool::new(true));
	} else {
		guild_atomic.unwrap().store(true, Ordering::Release);
	}
	std::mem::drop(lock);

	let typing = channel.start_typing(&ctx.http);

	let url_list = message.attachments.iter()
		.filter_map(|a| {
			// 少なくとも、width/heightが存在していれば画像である。
			if a.width.is_some() {
				Some(a.url.clone())
			} else {
				None
			}
		})
		// 今のところ、1つだけ受け付けるようにする。ケースに応じで変える
		.take(1)
		.collect();

	let user_message = create_user_message_with_image_url(
		message.content.clone(),
		calculate_likability_level_from_message_count(user_data.chat_message_count.unwrap_or_default()),
		user_data.uid,
		user_data.call_name.unwrap_or_default(),
		user_data.gender.unwrap_or(entity::enums::Gender::Ladies),
		&chrono::Local::now(),
		url_list,
	);

	let prev_id = GUILD_CHATTING_PREV_ID.lock().await.get(&guild_id).map(|v| v.clone());
	let (response_data, id) = getchat_responce(
		user_data.uid,
		user_message,
		prev_id,
	).await.inspect_err(|e| log::error!("{:?}", e)).unwrap();

	let mut mention_ids: Vec<UserId> = vec![];
	let mut is_mentioned_you = false;
	for action in &response_data.actions {
		if action.name == "send_message_channel" {
			if let Some(serde_json::Value::String(v)) = action.params.get("text") {
				message.channel_id
					.send_message(ctx, CreateMessage::new().content(v)).await.unwrap();
			}
		} else if let Some(serde_json::Value::Number(v)) = action.params.get("user_id") {
			if let Some(target_user_id) = v.as_u64().map(|v| UserId::new(v)) {
				if target_user_id == user.id {
					is_mentioned_you = true;
				} else {
					mention_ids.push(target_user_id);
				}
			}
		}
	}

	let mention_text = mention_ids.iter().map(|v| format!("{}", v.mention())).collect::<Vec<_>>().join(" ");
	let res_text = if mention_ids.len() == 0 {
		response_data.message
	} else {
		format!("{} {}", mention_text.trim(), response_data.message)
	};

	typing.stop();
	if is_mentioned_you {
		message.reply_ping(ctx, res_text).await.unwrap();
	} else {
		message.reply(ctx, res_text).await.unwrap();
	}

	GUILD_CHATTING_PREV_ID.lock().await.insert(guild_id, id);

	GUILD_CHATTING_STATE.lock().await.get(&guild_id).unwrap().store(false, Ordering::Release);
}

async fn check_send_target_channel_id(guild_id: GuildId, channel_id: ChannelId) -> bool {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();

	let guild_config = GuildConfigBehavior::find_by_id(guild_id.get()).one(mysql_client).await;
	std::mem::drop(lsc);

	if let Err(error) = guild_config {
		log::error!("DB Error: {:?}", error);
		return false;
	} else if let Ok(None) = guild_config {
		log::error!("not found guild config.");
		return false;
	}

	guild_config.unwrap().unwrap().send_ai_chat_channel_id.map(|v| v == channel_id.get()).unwrap_or(false)
}

async fn get_target_user_data(user_id: UserId) -> Option<UserData> {
	let lsc = STATIC_COMPONENTS.lock().await;
	let mysql_client = lsc.get_sql_client();

	let user_data = UserDataBehavior::find_by_id(user_id.get()).one(mysql_client).await;
	std::mem::drop(lsc);

	if let Err(error) = user_data {
		log::error!("DB Error: {:?}", error);
		return None;
	} else if let Ok(None) = user_data {
		log::error!("not found user data.");
		return None;
	}

	Some(user_data.unwrap().unwrap())
}
