use log::{error, info};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Member;
use serenity::model::id::GuildId;
use serenity::model::prelude::User;
use crate::events::{message_event, ready_event};

pub struct Router;

#[async_trait]
impl EventHandler for Router {
	async fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, new_member: Member) {
		info!("new member!");
		info!("  username: {}#{:04}", new_member.user.name, new_member.user.discriminator);
	}

	async fn guild_member_removal(&self, ctx: Context, _guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
		info!("member removed");
		info!("  username: {}#{:04}", user.name, user.discriminator);
		if let Some(member) = member_data_if_available {
			info!("member data found!");
			if let Err(error) = member.ban(&ctx.http, 0).await {
				error!("{}", error);
			}
		}
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
}
