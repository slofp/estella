use log::info;
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::GuildId;

pub async fn execute(ctx: Context, guild_id: GuildId, new_member: Member) {
	info!("new member!");
	info!("  username: {}#{:04}", new_member.user.name, new_member.user.discriminator);
}