use log::debug;
use serenity::client::Context;
use serenity::model::gateway::Ready;
use crate::utils::glacialeur;

pub async fn execute(ctx: Context, data_about_bot: Ready) {
	for guild in data_about_bot.guilds {
		let guildid = guild.id();
		debug!("id: {}", guildid.as_u64() );
		if *guildid.as_u64() == 0 /* my guild id */ {
			if let Ok(members) = guildid.members(&ctx.http, None, None).await {
				for member in members {
					if member.user.bot {
						continue;
					}
					let id = glacialeur::generate(
						member.user.id.as_u64(),
						1 << 3,
						member.joined_at.unwrap_or_else(|| guildid.created_at()).timestamp() - guildid.created_at().timestamp()
					);
					println!("{}#{:04}: {}", member.user.name, member.user.discriminator, id);
				}
			}

			break;
		}
	}
}
