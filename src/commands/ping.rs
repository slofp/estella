use crate::command_define::{BaseCommand, Command};
use crate::utils::color;
use chrono::{TimeDelta, Utc};
use log::error;
use serenity::all::{CommandDataOption, CommandInteraction, CreateInteractionResponse};
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateInteractionResponseMessage, EditInteractionResponse};
use serenity::client::Context;

pub struct PingCommand;

impl BaseCommand for PingCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"ping".into()
	}

	fn get_description(&self) -> String {
		"Botのpingを測ります".into()
	}
}

impl PingCommand {
	async fn defer_res(&self, ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
		command
			.create_response(
				&ctx.http,
				CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
			)
			.await
	}

	async fn get_ping(&self, ctx: &Context, command: &CommandInteraction) -> i64 {
		match command.get_response(&ctx.http).await {
			Ok(command_res) => {
				let ping_duration: TimeDelta =
					command_res.timestamp.with_timezone(&Utc) - command.id.created_at().with_timezone(&Utc);
				ping_duration.num_milliseconds()
			},
			Err(error) => {
				error!("{}", error);
				-1
			},
		}
	}

	async fn send_result(&self, ctx: &Context, command: &CommandInteraction, ping: i64) -> serenity::Result<()> {
		command
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new().add_embed(
					CreateEmbed::new()
						.title("Ping結果")
						.description(format!("{}ms", ping))
						.color(color::normal_color()),
				),
			)
			.await?;

		Ok(())
	}
}

#[async_trait]
impl Command for PingCommand {
	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		_: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let ctx = &ctx;
		let command = &command;
		self.defer_res(ctx, command).await?;

		self.send_result(ctx, command, self.get_ping(ctx, command).await).await
	}
}
