use crate::command_define::{BaseCommand, Command};
use crate::utils::color;
use serenity::all::{
	CommandDataOption, CommandInteraction, CreateEmbed, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use serenity::async_trait;
use serenity::client::Context;

const PROJECT_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct VersionCommand;

impl BaseCommand for VersionCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"version".into()
	}

	fn get_description(&self) -> String {
		"Estellaのバージョンを表示します".into()
	}
}

impl VersionCommand {
	fn get_project_name(&self) -> String {
		let mut p_name_vec: Vec<char> = PROJECT_NAME.chars().collect();
		p_name_vec[0] = p_name_vec[0].to_ascii_uppercase();
		return p_name_vec.iter().collect();
	}
}

#[async_trait]
impl Command for VersionCommand {
	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		_: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		command
			.create_response(
				&ctx.http,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new().add_embed(
						CreateEmbed::new()
							.title("バージョン情報")
							.description(format!("{} {}", self.get_project_name(), VERSION))
							.color(color::normal_color()),
					),
				),
			)
			.await
	}
}
