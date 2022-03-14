use log::error;
use serenity::builder::CreateApplicationCommandOption;
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType};
use serenity::model::interactions::InteractionResponseType;

pub async fn execute(ctx: Context, command: ApplicationCommandInteraction) {
	if let Err(error) = command.create_interaction_response(&ctx.http,
		|res|
			res.kind(InteractionResponseType::DeferredChannelMessageWithSource)
	).await {
		error!("{}", error);
		return;
	}

	let mut ping_ms: i64 = -1;
	match command.get_interaction_response(&ctx.http).await {
		Ok(command_res) => {
			let ping_duration = command_res.timestamp - command.id.created_at();
			ping_ms = ping_duration.num_milliseconds();
		},
		Err(error) => error!("{}", error)
	}

	if let Err(error) = command.edit_original_interaction_response(&ctx.http,
		|m|
			m.content(format!("{}ms", ping_ms))
	).await {
		error!("{}", error);
	}
}

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("ping")
		.description("Botのpingを測ります")
		.kind(ApplicationCommandOptionType::SubCommand)
}
