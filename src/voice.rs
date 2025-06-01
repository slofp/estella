use std::sync::Arc;

use receive::Receiver;
use serenity::all::{ChannelId, Context, GuildId};
use songbird::CoreEvent;

use crate::STATIC_COMPONENTS;

mod receive;
mod speak2text;
mod text_talk;
pub(crate) mod text2speak;

pub(crate) async fn connect_voice_channel(
	ctx: &Context,
	target_guild_id: &GuildId,
	target_channel_id: &ChannelId,
) -> Result<(), &'static str> {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice client placed in at initialisation.")
		.clone();

	{
		let handler_lock = manager.get_or_insert(target_guild_id.clone());
		let mut handler = handler_lock.lock().await;

		let comp_lock = STATIC_COMPONENTS.lock().await;
		let token = comp_lock.get_config().get_deepgram_token().clone();
		std::mem::drop(comp_lock);

		let mut evt_receiver = Receiver::new(Arc::clone(&ctx.http), Arc::clone(&handler_lock), token).await;
		evt_receiver.data.write().await.start();

		handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
		handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);
	}

	if let Ok(_) = manager.join(target_guild_id.clone(), target_channel_id.clone()).await {
		Ok(())
	} else {
		// Although we failed to join, we need to clear out existing event handlers on the call.
		_ = manager.remove(target_guild_id.clone()).await;

		Err("Error joining the channel")
	}
}

pub(crate) async fn disconnect_voice_channel(ctx: &Context, target_guild_id: &GuildId) -> Result<(), String> {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice client placed in at initialisation.")
		.clone();

	let has_handler = manager.get(target_guild_id.clone()).is_some();

	if has_handler {
		if let Err(e) = manager.remove(target_guild_id.clone()).await {
			Err(format!("エラー: {:?}", e))
		} else {
			Ok(())
		}
	} else {
		Err("どこにも接続していません。".to_string())
	}
}
