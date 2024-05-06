use log::error;
use serenity::all::{CommandInteraction, CommandOptionType};
use serenity::builder::CreateCommandOption;
use serenity::client::Context;
use crate::utils::color;

const PROJECT_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn get_project_name() -> String {
	let mut p_name_vec: Vec<char> = PROJECT_NAME.chars().collect();
	p_name_vec[0] = p_name_vec[0].to_ascii_uppercase();
	return p_name_vec.iter().collect();
}

pub async fn execute(ctx: Context, command: CommandInteraction) {
	if let Err(error) = command.create_interaction_response(&ctx.http,
		|res|
			res
				.kind(InteractionResponseType::ChannelMessageWithSource)
				.interaction_response_data(|m| {
					m
						.create_embed(|e| {
							e
								.title("バージョン情報")
								.description(format!("{} {}", get_project_name(), VERSION))
								.color(color::normal_color())
						})
				})
	).await {
		error!("{}", error);
		return;
	}
}

pub fn command_build() -> CreateCommandOption {
	CreateCommandOption::new(
		CommandOptionType::SubCommand,
		"version",
		"Estellaのバージョンを表示します"
	)
}
