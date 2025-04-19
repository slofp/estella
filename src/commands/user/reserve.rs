use crate::command_define::{BaseCommand, Command};
use crate::events::ready_event::ADD_PENDING_USERS;
use crate::utils::convert::flatten_result_option;
use crate::utils::{color, convert};
use crate::STATIC_COMPONENTS;
use chrono::{Duration, Utc};
use entity::enums::AccountType;
use entity::{
	confirmed_account, main_account, pending_account, sub_account, ConfirmedAccountBehavior,
	GuildConfigBehavior, MainAccountBehavior, PendingAccount, PendingAccountBehavior, SubAccountBehavior,
};
use log::error;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter};
use serenity::all::{
	ButtonStyle, CommandDataOption, CommandInteraction, CommandOptionType, CreateActionRow, CreateButton, CreateEmbed,
	CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse,
	InteractionResponseFlags,
};
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::id::ChannelId;
use std::num::ParseIntError;

const PARAM_USERID: &str = "user_id";
const PARAM_NAME: &str = "name";
const PARAM_REASON: &str = "reason";

/*
Paramsは値名→説明→型定義→必須で構成されています
*/
const PARAMS: [(&str, &str, CommandOptionType, bool); 3] = [
	(PARAM_USERID, "ユーザーID", CommandOptionType::String, true),
	(PARAM_NAME, "登録名", CommandOptionType::String, true),
	(PARAM_REASON, "登録理由", CommandOptionType::String, false),
];

pub struct ReserveCommand;

impl BaseCommand for ReserveCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"reserve".into()
	}

	fn get_description(&self) -> String {
		"ユーザー登録を予約します".into()
	}
}

#[async_trait]
impl Command for ReserveCommand {
	fn args_param(&self) -> &'static [(&'static str, &'static str, CommandOptionType, bool)] {
		&PARAMS
	}

	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		args: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let mut user_id: Option<String> = None;
		let mut name: Option<String> = None;
		let mut reason: Option<String> = None;

		for option in &args {
			//info!("option data: {} [{:?}]", b.name, b.value);
			match (&option.name).as_str() {
				PARAM_USERID => {
					let option_value = &option.value;
					if matches!(option_value.kind(), CommandOptionType::String) {
						user_id = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
					}
				},
				PARAM_NAME => {
					let option_value = &option.value;
					if matches!(option_value.kind(), CommandOptionType::String) {
						name = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
					}
				},
				PARAM_REASON => {
					let option_value = &option.value;
					if matches!(option_value.kind(), CommandOptionType::String) {
						reason = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
					}
				},
				_ => {},
			}
		}

		let mut error_message: Option<String> = None;

		if user_id.is_none() {
			error!("UserID is undefined.");
			error_message = Some("UserIDが入力されていません".to_string());
		} else if name.is_none() {
			error!("Name is undefined.");
			error_message = Some("Nameが入力されていません".to_string());
		}

		let mut user_id_r: Option<Result<u64, ParseIntError>> = None;
		if error_message.is_none() {
			user_id_r = Some(user_id.unwrap().parse::<u64>());
			if let Err(error) = user_id_r.as_ref().unwrap() {
				error!("user_id coundnt convert u64: {:?}", error);
				error_message = Some(format!("UserIDの記述が正しくありません: {:?}", error).to_string());
			}
		}

		if let Some(err_msg_f) = error_message {
			return command
				.create_response(
					&ctx.http,
					CreateInteractionResponse::Message(
						CreateInteractionResponseMessage::new().add_embed(
							CreateEmbed::new()
								.title("エラー")
								.description(err_msg_f)
								.color(color::failed_color()),
						),
					),
				)
				.await;
		}

		let user_id: u64 = user_id_r.unwrap().unwrap();
		let name: String = name.unwrap();

		command
			.create_response(
				&ctx.http,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new()
						.add_embed(
							CreateEmbed::new()
								.title("処理中...")
								.description("このメッセージを削除しないでください")
								.color(color::normal_color()),
						)
						.flags(InteractionResponseFlags::EPHEMERAL),
				),
			)
			.await?;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql_client();
		let check_user = MainAccountBehavior::find()
			.filter(main_account::Column::GuildId.eq(command.guild_id.unwrap().get()))
			.filter(main_account::Column::Uid.eq(user_id))
			.count(locked_db)
			.await
			.unwrap_or(0) +
			SubAccountBehavior::find()
				.filter(sub_account::Column::GuildId.eq(command.guild_id.unwrap().get()))
				.filter(sub_account::Column::Uid.eq(user_id))
				.count(locked_db)
				.await
				.unwrap_or(0) +
			ConfirmedAccountBehavior::find()
				.filter(confirmed_account::Column::GuildId.eq(command.guild_id.unwrap().get()))
				.filter(confirmed_account::Column::Uid.eq(user_id))
				.count(locked_db)
				.await
				.unwrap_or(0) +
			PendingAccountBehavior::find()
				.filter(pending_account::Column::GuildId.eq(command.guild_id.unwrap().get()))
				.filter(pending_account::Column::Uid.eq(user_id))
				.count(locked_db)
				.await
				.unwrap_or(0) >
			0;
		std::mem::drop(lsc);
		if check_user {
			command
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new().embeds(vec![CreateEmbed::new()
						.title("エラー")
						.description("すでに申請されているか登録されています")
						.color(color::failed_color())]),
				)
				.await?;
			return Ok(());
		}

		command
			.edit_response(
				&ctx.http,
				EditInteractionResponse::new()
					.embeds(vec![CreateEmbed::new()
						.title("確認")
						.description("以下の内容で登録します")
						.field("ユーザーID", user_id.to_string(), true)
						.field("名前", &name, true)
						.color(color::normal_color())])
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new(format!("ok_{}", &user_id))
							.style(ButtonStyle::Success)
							.label("OK"),
						CreateButton::new(format!("cancel_{}", &user_id))
							.style(ButtonStyle::Danger)
							.label("キャンセル"),
					])]),
			)
			.await?;

		let message = command.get_response(&ctx.http).await.unwrap();
		let button_interaction = match message
			.await_component_interaction(&ctx)
			.timeout(std::time::Duration::from_secs(60 * 3))
			.await
		{
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return Ok(());
			},
		};

		if button_interaction.data.custom_id == format!("ok_{}", &user_id) {
			/*if let Err(error) = message.delete(&ctx.http).await {
				error!("{}", error);
				return;
			}*/
			button_interaction.defer(&ctx.http).await?;

			button_interaction
				.edit_response(
					&ctx.http,
					EditInteractionResponse::new()
						.components(vec![])
						.embeds(vec![CreateEmbed::new()
							.title("処理中...")
							.description("このメッセージを削除しないでください")
							.color(color::normal_color())]),
				)
				.await?;

			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			if command.guild_id.is_none() {
				error!("Should not occur error: GuildID is none");
				return Ok(());
			}
			let guild_config = match flatten_result_option(
				GuildConfigBehavior::find_by_id(command.guild_id.unwrap().get())
					.one(locked_db)
					.await,
			) {
				Ok(x) => Some(x),
				Err(e) => {
					error!("DB Error: {:?}", e);
					error_message = Some(format!("{:?}", e));
					None
				},
			};
			if let Some(guild_config) = guild_config {
				if let Some(guild_log_channel) = guild_config.log_channel_id {
					let end_vote_time = Utc::now() + Duration::days(7);
					let log_channel = ChannelId::new(guild_log_channel);
					let vote_message = log_channel.send_message(&ctx.http,
						CreateMessage::new()
							.add_embed({
								let e = CreateEmbed::new()
									.title("追加申請")
									.description("以下の内容で登録申請されました。内容を見てこのサーバーに入れたくないと判断した場合は「却下」ボタンを押してください")
									.field("ユーザーID", user_id.to_string(), true)
									.field("名前", &name, true)
									.field("申請却下終了時刻", convert::utc_to_local_format(&end_vote_time), true)
									.color(color::normal_color());
								if let Some(reason) = reason {
									e.field("申請理由", reason, true)
								} else {
									e
								}
							})
							.components(vec![
								CreateActionRow::Buttons(vec![
									CreateButton::new(format!("reject_{}", &user_id)).style(ButtonStyle::Danger).label("申請を却下する")
								])
							])
					).await?;

					let pending_data = PendingAccount {
						uid: user_id,
						name: Some(name.clone()),
						message_id: vote_message.id.get(),
						end_voting: Some(Utc::now() + Duration::days(7) /*Duration::seconds(30)*/),
						guild_id: guild_config.uid,
						account_type: AccountType::Main,
						main_uid: None,
						first_cert: None,
					};

					let pending_data = pending_data.into_active_model().insert(locked_db).await;
					if let Err(error) = pending_data {
						error!("DB Error: {:?}", error);
						error_message = Some(format!("{:?}", error));
					} else {
						let mut lpu = ADD_PENDING_USERS.lock().await;
						lpu.push(pending_data.unwrap());
						std::mem::drop(lpu);
					}
				} else {
					error!("Error: Not found log channel");
					error_message = Some(String::from("ログチャンネルが指定されていません"));
				}
			}

			std::mem::drop(lsc);

			button_interaction
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
								.description("以下の内容で登録しました！7日間何も無ければ正式に登録されます")
								.field("ユーザーID", user_id.to_string(), true)
								.field("名前", &name, true)
								.color(color::success_color())
						},
					]),
				)
				.await?;
			//button_interaction.defer(&ctx.http).await;
		} else if button_interaction.data.custom_id == format!("cancel_{}", &user_id) {
			/*if let Err(error) = message.delete(&ctx.http).await {
				error!("{}", error);
				return;
			}*/
			button_interaction
				.create_response(
					&ctx.http,
					CreateInteractionResponse::UpdateMessage(
						CreateInteractionResponseMessage::new()
							.components(vec![])
							.embeds(vec![CreateEmbed::new()
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())]),
					),
				)
				.await?;
			//button_interaction.defer(&ctx.http).await;
		}

		Ok(())
	}
}
