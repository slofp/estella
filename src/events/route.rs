use std::collections::HashMap;
use log::info;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;
use serenity::http::CacheHttp;
use serenity::model::channel::{Channel, ChannelCategory, GuildChannel, Message, MessageType, PartialGuildChannel, Reaction, StageInstance};
use serenity::model::event::{ChannelPinsUpdateEvent, GuildMembersChunkEvent, GuildMemberUpdateEvent, InviteCreateEvent, InviteDeleteEvent, MessageUpdateEvent, PresenceUpdateEvent, ResumedEvent, ThreadListSyncEvent, ThreadMembersUpdateEvent, TypingStartEvent, VoiceServerUpdateEvent};
use serenity::model::gateway::{Presence, Ready};
use serenity::model::guild::{Emoji, Guild, GuildUnavailable, Integration, Member, PartialGuild, Role, ThreadMember};
use serenity::model::id::{ApplicationId, ChannelId, EmojiId, GuildId, IntegrationId, MessageId, RoleId};
use serenity::model::prelude::{CurrentUser, User, VoiceState};
use serenity::model::user::OnlineStatus;
use crate::events::{message_event, ready_event};
use crate::utils::glacialeur;

pub struct Router;

#[async_trait]
impl EventHandler for Router {
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
