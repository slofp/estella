use log::info;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::interactions::Interaction;
use serenity::model::prelude::User;
use crate::events::{interaction_event, member_add_event, member_remove_event, message_event, message_remove_event, ready_event};

pub struct Router;

#[async_trait]
impl EventHandler for Router {
	async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
		info!("Guild Member Add event start");
		member_add_event::execute(ctx, guild_id, new_member).await;
		info!("Guild Member Add event end");
	}

	async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
		info!("Guild Member Remove event start");
		member_remove_event::execute(ctx, guild_id, user, member_data_if_available).await;
		info!("Guild Member Remove event end");
	}

	async fn message(&self, ctx: Context, message: Message) {
		info!("Message created event start");
		message_event::execute(ctx, message).await;
		info!("Message created event end");
	}

	async fn ready(&self, ctx: Context, data_about_bot: Ready) {
		info!("Ready event start");
		ready_event::execute(ctx, data_about_bot).await;
		info!("Ready event end");
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		info!("Message Interaction created event start");
		interaction_event::execute(ctx, interaction).await;
		info!("Message Interaction created event end");
	}

	async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
		info!("Message removed event start");
		message_remove_event::execute(ctx, channel_id, deleted_message_id, guild_id).await;
		info!("Message removed event end");
	}
}
