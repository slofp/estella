use log::error;
use serenity::all::{CommandDataOption, CommandInteraction, CreateCommandOption, CreateInteractionResponse};
use serenity::builder::{CreateEmbed, CreateInteractionResponseMessage, EditInteractionResponse};
use serenity::client::Context;
use crate::command_define::{BaseCommand, Command};
use crate::utils::color;

pub struct PingCommand;

impl BaseCommand for PingCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> impl Into<String> {
		"ping"
	}

	fn get_description(&self) -> impl Into<String> {
		"Botのpingを測ります"
	}
}

impl PingCommand {
	async fn defer_res(&self, ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
		command.create_response(&ctx.http,
			CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new())
		).await
	}

	async fn get_ping(&self, ctx: &Context, command: &CommandInteraction) -> i64 {
		match command.get_response(&ctx.http).await {
			Ok(command_res) => {
				let ping_duration = command_res.timestamp - command.id.created_at();
				ping_duration.num_milliseconds()
			},
			Err(error) => {
				error!("{}", error);
				-1
			}
		}
	}

	async fn send_result(&self, ctx: &Context, command: &CommandInteraction, ping: i64) -> serenity::Result<()> {
		command.edit_response(&ctx.http,
			EditInteractionResponse::new()
				.add_embed(
					CreateEmbed::new()
						.title("Ping結果")
						.description(format!("{}ms", ping))
						.color(color::normal_color())
				)
		).await?;

		Ok(())
	}
}

impl Command for PingCommand {
	async fn execute(&self, ctx: Context, command: CommandInteraction, args: Vec<CommandDataOption>) -> serenity::Result<()>  {
		let ctx = &ctx;
		let command = &command;
		self.defer_res(ctx, command).await?;

		self.send_result(ctx, command, self.get_ping(ctx, command).await).await
	}
}
