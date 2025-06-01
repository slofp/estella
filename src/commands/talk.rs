use crate::command_define::{BaseCommand, Command};
use crate::utils::color;
use crate::voice::connect_voice_channel;
use serenity::all::{
	ChannelType, CommandDataOption, CommandInteraction, CreateEmbed, CreateInteractionResponse,
	CreateInteractionResponseMessage, GuildChannel,
};
use serenity::client::Context;
use serenity::{async_trait, Error};

pub struct TalkCommand;

impl BaseCommand for TalkCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"talk".into()
	}

	fn get_description(&self) -> String {
		"実行したユーザーの居るVCに接続します".into()
	}
}

impl TalkCommand {
	async fn get_user_connected_voice_channel(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
	) -> serenity::Result<GuildChannel> {
		let channels: Vec<GuildChannel> = command
			.guild_id
			.unwrap()
			.channels(ctx)
			.await?
			.into_iter()
			.filter_map(|v| {
				let channel = v.1;
				if channel.kind == ChannelType::Voice {
					Some(channel)
				} else {
					None
				}
			})
			.collect();

		let send_member = command.member.as_ref().unwrap();

		for channel in channels {
			let connected_members = channel.members(ctx)?;

			if connected_members
				.iter()
				.any(|v| v.user.id.get() == send_member.user.id.get())
			{
				return Ok(channel);
			}
		}

		self.send_not_connected_voice_channel(ctx, command).await?;

		Err(Error::Other("send member connected voice channel not found."))
	}

	async fn send_not_connected_voice_channel(
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
									.title("接続に失敗")
									.description("あなたはどのVCにも接続していません。\nコマンドを使用するには、VCに接続してから実行してください。")
									.color(color::failed_color()),
							)
					),
			)
			.await
	}

	async fn send_error_connecting_voice_channel(
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
									.title("接続に失敗")
									.description("VCに接続しようとした際にエラーが発生し接続できませんでした。")
									.color(color::failed_color()),
							)
					),
			)
			.await
	}

	async fn send_connected_voice_channel(
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
									.title("接続完了")
									.description("VCに接続しました！")
									.color(color::success_color()),
							)
					),
			)
			.await
	}
}

#[async_trait]
impl Command for TalkCommand {
	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		_: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let ctx = &ctx;
		let command = &command;
		let channel = self.get_user_connected_voice_channel(ctx, command).await?.id;
		let guild = command.guild_id.unwrap();

		if let Err(_) = connect_voice_channel(ctx, &guild, &channel).await {
			self.send_error_connecting_voice_channel(ctx, command).await
		} else {
			self.send_connected_voice_channel(ctx, command).await
		}
	}
}
