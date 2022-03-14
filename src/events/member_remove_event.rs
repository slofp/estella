use log::{error, info};
use serenity::client::Context;
use serenity::model::guild::Member;
use serenity::model::id::GuildId;
use serenity::model::user::User;

pub async fn execute(ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
	info!("member removed");
	info!("  username: {}#{:04}", user.name, user.discriminator);
	if let Some(member) = member_data_if_available {
		info!("member data found!");
		if let Err(error) = member.ban(&ctx.http, 0).await {
			error!("{}", error);
		}
	}
}