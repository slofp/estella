use std::sync::Arc;

use receive::Receiver;
use serenity::all::{ChannelId, Context, GuildId};
use songbird::{CoreEvent, Songbird};

use crate::STATIC_COMPONENTS;

mod receive;
mod speak2text;
mod text_talk;
pub(crate) mod text2speak;

pub(crate) async fn connect_voice_channel(
	ctx: &Context,
	target_guild_id: GuildId,
	target_channel_id: ChannelId,
) -> Result<(), &'static str> {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice client placed in at initialisation.")
		.clone();

	{
		let handler_lock = manager.get_or_insert(target_guild_id);
		let mut handler = handler_lock.lock().await;

		let comp_lock = STATIC_COMPONENTS.lock().await;
		let token = comp_lock.get_config().get_deepgram_token().clone();
		std::mem::drop(comp_lock);

		let evt_receiver = Receiver::new(
			Arc::downgrade(&manager),
			target_guild_id,
			Arc::clone(&ctx.http),
			Arc::downgrade(&handler_lock),
			token
		).await;
		evt_receiver.data.write().await.start(target_guild_id).await;

		handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
		handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver.clone());
		handler.add_global_event(CoreEvent::ClientDisconnect.into(), evt_receiver);
	}

	if let Ok(_) = manager.join(target_guild_id, target_channel_id).await {
		Ok(())
	} else {
		// Although we failed to join, we need to clear out existing event handlers on the call.
		_ = manager.remove(target_guild_id.clone()).await;

		Err("Error joining the channel")
	}
}

pub(crate) async fn disconnect_voice_channel(ctx: &Context, target_guild_id: GuildId) -> Result<(), String> {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice client placed in at initialisation.")
		.clone();

	disconnect_voice_channel_from_manager(&manager, target_guild_id).await
}

pub(crate) async fn disconnect_voice_channel_from_manager(manager: &Arc<Songbird>, target_guild_id: GuildId) -> Result<(), String> {
	let has_handler = manager.get(target_guild_id).is_some();

	if has_handler {
		if let Err(e) = manager.remove(target_guild_id).await {
			Err(format!("エラー: {:?}", e))
		} else {
			Ok(())
		}
	} else {
		Err("どこにも接続していません。".to_string())
	}
}
