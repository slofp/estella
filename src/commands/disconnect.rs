use crate::command_define::{BaseCommand, Command};
use crate::utils::color;
use crate::voice::disconnect_voice_channel;
use serenity::all::{
	CommandDataOption, CommandInteraction, CreateEmbed, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use serenity::client::Context;
use serenity::async_trait;

pub struct DisconnectCommand;

impl BaseCommand for DisconnectCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"disconnect".into()
	}

	fn get_description(&self) -> String {
		"VCから切断します".into()
	}
}

impl DisconnectCommand {
	async fn send_error_disconnect_voice_channel(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		msg: String,
	) -> serenity::Result<()> {
		command
			.create_response(
				&ctx,
				CreateInteractionResponse::Message(
						CreateInteractionResponseMessage::new()
							.add_embed(
								CreateEmbed::new()
									.title("エラー")
									.description(msg)
									.color(color::failed_color()),
							)
					),
			)
			.await
	}

	async fn send_disconnected_voice_channel(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
	) -> serenity::Result<()> {
		command
			.create_response(
				&ctx,
				CreateInteractionResponse::Message(
						CreateInteractionResponseMessage::new()
							.add_embed(
								CreateEmbed::new()
									.title("完了")
									.description("VCから切断しました！")
									.color(color::success_color()),
							)
					),
			)
			.await
	}
}

#[async_trait]
impl Command for DisconnectCommand {
	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		_: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let ctx = &ctx;
		let command = &command;
		let guild = command.guild_id.unwrap();

		if let Err(msg) = disconnect_voice_channel(ctx, guild).await {
			self.send_error_disconnect_voice_channel(ctx, command, msg).await
		} else {
			self.send_disconnected_voice_channel(ctx, command).await
		}
	}
}
