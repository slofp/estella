use crate::command_define::{BaseCommand, Command};
use crate::utils::color;
use crate::STATIC_COMPONENTS;
use entity::guild_config::ActiveModel as GuildConfigActiveModel;
use log::error;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ActiveValue};
use serenity::all::{
	ButtonStyle, CommandDataOption, CommandInteraction, ComponentInteraction, ComponentInteractionDataKind,
	CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
	CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, InteractionResponseFlags,
};
use serenity::async_trait;
use serenity::client::Context;

const MENU_PARAM_LC: &str = "log_channel";
const MENU_PARAM_AR: &str = "auth_role";
const MENU_PARAM_BR: &str = "bot_role";
const MENU_PARAM_WL: &str = "white_list";
const MENU_PARAM_LB: &str = "leave_ban";

pub struct ConfigCommand;

impl BaseCommand for ConfigCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"config".into()
	}

	fn get_description(&self) -> String {
		"Botの設定をします".into()
	}
}

impl ConfigCommand {
	async fn log_channel_config(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		select_interaction: ComponentInteraction,
	) {
		let edit_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.components(vec![])
					.embeds(vec![CreateEmbed::new()
						.title("ログチャンネル設定")
						.description("ログチャンネルにするチャンネルのIDを入力してください。")
						.color(color::normal_color())]),
			)
			.await;
		if let Err(error) = edit_message {
			error!("Error: {}", error);
			return;
		}
		let edit_message = edit_message.unwrap();

		let rep_message = match edit_message
			.channel_id
			.await_reply(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			None => {
				error!("Wait message timeout...");
				return;
			},
			Some(x) => x,
		};

		if let Err(error) = rep_message.delete(&ctx.http).await {
			error!("{:?}", error);
		}

		let mut error_message: Option<String> = None;
		let channel_id_r = rep_message.content.clone().parse::<u64>();
		if let Err(ref error) = channel_id_r {
			error!("channel_id coundnt convert u64: {:?}", error);
			error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
		}
		if let Some(error_message) = error_message {
			if let Err(error) = select_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new()
						.components(vec![])
						.embeds(vec![CreateEmbed::new()
							.title("エラー")
							.description(error_message)
							.color(color::failed_color())]),
				)
				.await
			{
				error!("Error: {}", error);
			}
			return;
		}
		let channel_id: u64 = channel_id_r.unwrap();

		let confirm_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![CreateEmbed::new()
						.title("確認")
						.description("以下の内容で設定します")
						.field("チャンネルID", channel_id.to_string(), true)
						.color(color::normal_color())])
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("ok_{}", &channel_id))
							.style(ButtonStyle::Success)
							.label("OK"),
						CreateButton::new(format!("cancel_{}", &channel_id))
							.style(ButtonStyle::Danger)
							.label("キャンセル"),
					])]),
			)
			.await;
		if let Err(error) = confirm_message {
			error!("Error: {}", error);
			return;
		}

		let button_interaction = match confirm_message
			.unwrap()
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			},
		};

		if button_interaction.data.custom_id == format!("ok_{}", &channel_id) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			error_message = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.log_channel_id = ActiveValue::Set(Some(channel_id));
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("以下の内容で設定しました！")
								.field("チャンネルID", &channel_id.to_string(), true)
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else {
			if let Err(error) = button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())])
							.flags(InteractionResponseFlags::EPHEMERAL),
					),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		}
	}

	async fn auth_role_config(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		select_interaction: ComponentInteraction,
	) {
		let edit_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.components(vec![])
					.embeds(vec![CreateEmbed::new()
						.title("認証ロール設定")
						.description("承認したときに使用するロールのIDを入力してください。")
						.color(color::normal_color())]),
			)
			.await;
		if let Err(error) = edit_message {
			error!("Error: {}", error);
			return;
		}
		let edit_message = edit_message.unwrap();

		let rep_message = match edit_message
			.channel_id
			.await_reply(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			None => {
				error!("Wait message timeout...");
				return;
			},
			Some(x) => x,
		};

		if let Err(error) = rep_message.delete(&ctx.http).await {
			error!("{:?}", error);
		}

		let mut error_message: Option<String> = None;
		let auth_id_r = rep_message.content.clone().parse::<u64>();
		if let Err(ref error) = auth_id_r {
			error!("auth_id coundnt convert u64: {:?}", error);
			error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
		}
		if let Some(error_message) = error_message {
			if let Err(error) = select_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new()
						.components(vec![])
						.embeds(vec![CreateEmbed::new()
							.title("エラー")
							.description(error_message)
							.color(color::failed_color())]),
				)
				.await
			{
				error!("Error: {}", error);
			}
			return;
		}
		let auth_id: u64 = auth_id_r.unwrap();

		let confirm_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![CreateEmbed::new()
						.title("確認")
						.description("以下の内容で設定します")
						.field("ロールID", auth_id.to_string(), true)
						.color(color::normal_color())])
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("ok_{}", &auth_id))
							.style(ButtonStyle::Success)
							.label("OK"),
						CreateButton::new(format!("cancel_{}", &auth_id))
							.style(ButtonStyle::Danger)
							.label("キャンセル"),
					])]),
			)
			.await;
		if let Err(error) = confirm_message {
			error!("Error: {}", error);
			return;
		}

		let button_interaction = match confirm_message
			.unwrap()
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			},
		};

		if button_interaction.data.custom_id == format!("ok_{}", &auth_id) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			error_message = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.auth_role_id = ActiveValue::Set(Some(auth_id));
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("以下の内容で設定しました！")
								.field("ロールID", auth_id.to_string(), true)
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else {
			if let Err(error) = button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())])
							.flags(InteractionResponseFlags::EPHEMERAL),
					),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		}
	}

	async fn bot_role_config(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		select_interaction: ComponentInteraction,
	) {
		let edit_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.components(vec![])
					.embeds(vec![CreateEmbed::new()
						.title("Botロール設定")
						.description("Botがサーバーに入った時に使用するロールのIDを入力してください。")
						.color(color::normal_color())]),
			)
			.await;
		if let Err(error) = edit_message {
			error!("Error: {}", error);
			return;
		}
		let edit_message = edit_message.unwrap();

		let rep_message = match edit_message
			.channel_id
			.await_reply(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			None => {
				error!("Wait message timeout...");
				return;
			},
			Some(x) => x,
		};

		if let Err(error) = rep_message.delete(&ctx.http).await {
			error!("{:?}", error);
		}

		let mut error_message: Option<String> = None;
		let bot_id_r = rep_message.content.clone().parse::<u64>();
		if let Err(ref error) = bot_id_r {
			error!("auth_id coundnt convert u64: {:?}", error);
			error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
		}
		if let Some(error_message) = error_message {
			if let Err(error) = select_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new()
						.components(vec![])
						.embeds(vec![CreateEmbed::new()
							.title("エラー")
							.description(error_message)
							.color(color::failed_color())]),
				)
				.await
			{
				error!("Error: {}", error);
			}
			return;
		}
		let bot_id: u64 = bot_id_r.unwrap();

		let confirm_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![CreateEmbed::new()
						.title("確認")
						.description("以下の内容で設定します")
						.field("ロールID", bot_id.to_string(), true)
						.color(color::normal_color())])
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("ok_{}", &bot_id))
							.style(ButtonStyle::Success)
							.label("OK"),
						CreateButton::new(format!("cancel_{}", &bot_id))
							.style(ButtonStyle::Danger)
							.label("キャンセル"),
					])]),
			)
			.await;
		if let Err(error) = confirm_message {
			error!("Error: {}", error);
			return;
		}

		let button_interaction = match confirm_message
			.unwrap()
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			},
		};

		if button_interaction.data.custom_id == format!("ok_{}", &bot_id) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			error_message = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.bot_role_id = ActiveValue::Set(Some(bot_id));
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("以下の内容で設定しました！")
								.field("ロールID", bot_id.to_string(), true)
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else {
			if let Err(error) = button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())])
							.flags(InteractionResponseFlags::EPHEMERAL),
					),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		}
	}

	async fn white_list_config(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		select_interaction: ComponentInteraction,
	) {
		let confirm_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![])
					.add_embed(
						CreateEmbed::new()
							.title("ホワイトリスト設定")
							.description("このサーバーをホワイトリスト制御しますか？")
							.color(color::normal_color()),
					)
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("yes_{}", command.user.id.get()))
							.style(ButtonStyle::Success)
							.label("はい"),
						CreateButton::new(format!("no_{}", command.user.id.get()))
							.style(ButtonStyle::Danger)
							.label("いいえ"),
						CreateButton::new(format!("cancel_{}", command.user.id.get()))
							.style(ButtonStyle::Secondary)
							.label("キャンセル"),
					])]),
			)
			.await;
		if let Err(error) = confirm_message {
			error!("Error: {}", error);
			return;
		}

		let button_interaction = match confirm_message
			.unwrap()
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			},
		};

		if button_interaction.data.custom_id == format!("yes_{}", command.user.id.get()) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			let mut error_message: Option<String> = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.white_list = ActiveValue::Set(true);
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("ホワイトリスト制御を有効にしました！")
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else if button_interaction.data.custom_id == format!("no_{}", command.user.id.get()) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			let mut error_message: Option<String> = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.white_list = ActiveValue::Set(false);
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("ホワイトリスト制御を無効にしました！")
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else {
			if let Err(error) = button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())])
							.flags(InteractionResponseFlags::EPHEMERAL),
					),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		}
	}

	async fn leave_ban_config(
		&self,
		ctx: &Context,
		command: &CommandInteraction,
		select_interaction: ComponentInteraction,
	) {
		let confirm_message = select_interaction
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![CreateEmbed::new()
						.title("退出時BAN設定")
						.description("このサーバーの退出時BANを有効にしますか？")
						.color(color::normal_color())])
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("yes_{}", command.user.id.get()))
							.style(ButtonStyle::Success)
							.label("はい"),
						CreateButton::new(format!("no_{}", command.user.id.get()))
							.style(ButtonStyle::Danger)
							.label("いいえ"),
						CreateButton::new(format!("cancel_{}", command.user.id.get()))
							.style(ButtonStyle::Secondary)
							.label("キャンセル"),
					])]),
			)
			.await;
		if let Err(error) = confirm_message {
			error!("Error: {}", error);
			return;
		}

		let button_interaction = match confirm_message
			.unwrap()
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			},
		};

		if button_interaction.data.custom_id == format!("yes_{}", command.user.id.get()) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			let mut error_message: Option<String> = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.leave_ban = ActiveValue::Set(true);
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("退出時BANを有効にしました！")
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else if button_interaction.data.custom_id == format!("no_{}", command.user.id.get()) {
			if let Err(error) = button_interaction.defer(&ctx.http).await {
				error!("{}", error);
				return;
			}

			let mut error_message: Option<String> = None;
			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let mut update_entity = GuildConfigActiveModel::new();
			update_entity.uid = ActiveValue::Set(command.guild_id.unwrap().get());
			update_entity.leave_ban = ActiveValue::Set(false);
			if let Err(error) = update_entity.update(locked_db).await {
				error!("{:?}", error);
				error_message = Some(format!("{:?}", error));
			}
			std::mem::drop(lsc);

			if let Err(error) = button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().components(vec![]).embeds(vec![
						if let Some(err_msg) = error_message {
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg)
								.color(color::failed_color())
						} else {
							CreateEmbed::new()
								.title("完了")
								.description("退出時BANを無効にしました！")
								.color(color::success_color())
						},
					]),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		} else {
			if let Err(error) = button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())])
							.flags(InteractionResponseFlags::EPHEMERAL),
					),
				)
				.await
			{
				error!("{}", error);
				return;
			}
		}
	}
}

#[async_trait]
impl Command for ConfigCommand {
	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		_: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let guild = command.guild_id.unwrap().to_guild_cached(&ctx.cache).map(|v| v.clone());
		if let Some(guild) = guild {
			if command.user.id != guild.owner_id {
				return command
					.create_response(
						&ctx.http,
						CreateInteractionResponse::Message(
							CreateInteractionResponseMessage::new().add_embed(
								CreateEmbed::new()
									.title("エラー")
									.description("このコマンドはサーバーオーナーのみ使用できます")
									.color(color::failed_color()),
							),
						),
					)
					.await;
			}
		} else {
			error!("Not found Guild");
			return Ok(());
		}

		command
			.create_response(
				&ctx.http,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new()
						.components(vec![CreateActionRow::SelectMenu(
							CreateSelectMenu::new(
								format!("c-config_{}", command.user.id.get()),
								CreateSelectMenuKind::String {
									options: vec![
										CreateSelectMenuOption::new("ログチャンネル設定", MENU_PARAM_LC)
											.description("ログチャンネルの設定をします"),
										CreateSelectMenuOption::new("認証ロール設定", MENU_PARAM_AR)
											.description("認証ロールの設定をします"),
										CreateSelectMenuOption::new("Botロール設定", MENU_PARAM_BR)
											.description("Botロールの設定をします"),
										CreateSelectMenuOption::new("ホワイトリスト設定", MENU_PARAM_WL)
											.description("ホワイトリスト設定をします"),
										CreateSelectMenuOption::new("退鯖BAN設定", MENU_PARAM_LB)
											.description("サーバーを抜けたときの設定をします"),
									],
								},
							)
							.placeholder("設定したい項目を選択してください")
							.min_values(1)
							.max_values(1),
						)])
						.add_embed(
							CreateEmbed::new()
								.title("設定")
								.description("下の選択メニューから設定したい項目を選択してください。")
								.color(color::normal_color()),
						)
						.flags(InteractionResponseFlags::EPHEMERAL),
				),
			)
			.await?;

		let message = command.get_response(&ctx.http).await.unwrap();
		let select_interaction = message
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await;
		if select_interaction.is_none() {
			error!("interaction timeout...");
			return Ok(());
		}
		let select_interaction = select_interaction.unwrap();

		select_interaction.defer(&ctx.http).await?;

		if let ComponentInteractionDataKind::StringSelect { values } = &select_interaction.data.kind {
			for menu_value in values {
				match menu_value.as_str() {
					MENU_PARAM_LC => {
						self.log_channel_config(&ctx, &command, select_interaction.clone())
							.await
					},
					MENU_PARAM_AR => self.auth_role_config(&ctx, &command, select_interaction.clone()).await,
					MENU_PARAM_BR => self.bot_role_config(&ctx, &command, select_interaction.clone()).await,
					MENU_PARAM_WL => self.white_list_config(&ctx, &command, select_interaction.clone()).await,
					MENU_PARAM_LB => self.leave_ban_config(&ctx, &command, select_interaction.clone()).await,
					_ => {},
				}
			}
		}

		Ok(())
	}
}
