use crate::command_define::{BaseCommand, Command};
use crate::utils::convert::{flatten_result_option, utc_to_local_format};
use crate::utils::{color, convert};
use crate::STATIC_COMPONENTS;
use entity::enums::AccountType;
use entity::UserDataBehavior;
use log::error;
use sea_orm::EntityTrait;
use serenity::all::{
	CommandDataOption, CommandInteraction, CommandOptionType, CreateEmbed, CreateInteractionResponse,
	CreateInteractionResponseMessage, InteractionResponseFlags,
};
use serenity::async_trait;
use serenity::client::Context;
use std::num::ParseIntError;

const PARAM_USERID: &str = "user_id";

/*
Paramsは値名→説明→型定義→必須で構成されています
*/
const PARAMS: [(&str, &str, CommandOptionType, bool); 1] =
	[(PARAM_USERID, "ユーザーID", CommandOptionType::String, false)];

pub struct FindCommand;

impl BaseCommand for FindCommand {
	fn new() -> Self {
		Self {}
	}

	fn get_name(&self) -> String {
		"find".into()
	}

	fn get_description(&self) -> String {
		"ユーザー情報を表示します".into()
	}
}

#[async_trait]
impl Command for FindCommand {
	fn args_param(&self) -> &'static [(&'static str, &'static str, CommandOptionType, bool)] {
		&PARAMS
	}

	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		args: Vec<CommandDataOption>,
	) -> serenity::Result<()> {
		let mut user_id_o: Option<String> = None;

		for option in &args {
			//info!("option data: {} [{:?}]", b.name, b.value);
			match (&option.name).as_str() {
				PARAM_USERID => {
					let option_value = &option.value;
					if matches!(option_value.kind(), CommandOptionType::String) {
						user_id_o = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
					}
				},
				_ => {},
			}
		}

		let mut error_message: Option<String> = None;

		let mut user_id = command.user.id.get();
		if user_id_o.is_none() {
			error!("UserID is undefined.");
		} else {
			let user_id_r: Result<u64, ParseIntError> = user_id_o.unwrap().parse::<u64>();
			if let Err(error) = user_id_r {
				error!("user_id coundnt convert u64: {:?}", error);
				error_message = Some("UserIDの記述が正しくありません".to_string());
			} else {
				user_id = user_id_r.unwrap();
			}
		}

		if error_message.is_none() {
			let user_mem = command.guild_id.unwrap().member(&ctx.http, user_id).await;
			if let Err(error) = user_mem {
				error!("Error: {}", error);
				error_message = Some(format!("IDが見つからないかその他のエラーです: {}", error));
			} else {
				let user_mem = user_mem.unwrap();

				let lsc = STATIC_COMPONENTS.lock().await;
				let locked_db = lsc.get_sql_client();
				let user_data = flatten_result_option(UserDataBehavior::find_by_id(user_id).one(locked_db).await);
				std::mem::drop(lsc);
				if let Err(error) = user_data {
					error!("DB Error: {:?}", error);
					error_message = Some(format!("IDが見つからないかその他のエラーです: {:?}", error));
				} else {
					let user_data = user_data.unwrap();

					return command
						.create_response(
							&ctx.http,
							CreateInteractionResponse::Message(
								CreateInteractionResponseMessage::new()
									.add_embed(
										CreateEmbed::new()
											.title("ユーザー情報")
											.field("ID", user_data.uid.to_string(), true)
											.field("名前", convert::format_discord_username(&user_mem.user), true)
											.field(
												"アカウント作成日",
												utc_to_local_format(&user_mem.user.created_at()),
												true,
											)
											.field(
												"サーバー入鯖日",
												utc_to_local_format(
													&user_mem
														.joined_at
														.unwrap_or(command.guild_id.unwrap().created_at()),
												),
												true,
											)
											.field(
												"アカウントタイプ",
												(if user_data.glacialeur.is_none() {
													AccountType::Sub
												} else {
													AccountType::Main
												})
												.to_string(),
												true,
											)
											.field(
												"Glacialeur",
												user_data.glacialeur.unwrap_or("なし".to_string()),
												true,
											)
											.thumbnail(user_mem.user.avatar_url().unwrap_or("".to_string()))
											.color(user_mem.user.accent_colour.unwrap_or(color::normal_color())),
									)
									.flags(InteractionResponseFlags::EPHEMERAL),
							),
						)
						.await;
				}
			}
		}

		command
			.create_response(
				&ctx.http,
				CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new()
						.add_embed(
							CreateEmbed::new()
								.title("エラー")
								.description(error_message.unwrap())
								.color(color::failed_color()),
						)
						.flags(InteractionResponseFlags::EPHEMERAL),
				),
			)
			.await
	}
}
