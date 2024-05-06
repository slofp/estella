use std::num::ParseIntError;
use log::error;
use serenity::all::CommandInteraction;
use serenity::client::Context;
use crate::STATIC_COMPONENTS;
use crate::utils::{color, convert};
use crate::utils::convert::utc_to_local_format;
use entity::enums::AccountType;

const PARAM_USERID: &str = "user_id";

/*
Paramsは値名→説明→型定義→必須で構成されています
*/
const PARAMS: [(&str, &str, ApplicationCommandOptionType, bool); 1] = [
	(PARAM_USERID, "ユーザーID", ApplicationCommandOptionType::String, false)
];

pub async fn execute(ctx: Context, command: &CommandInteraction, command_args: &ApplicationCommandInteractionDataOption) {
	let mut user_id_o: Option<String> = None;

	for option in &command_args.options {
		//info!("option data: {} [{:?}]", b.name, b.value);
		match (&option.name).as_str() {
			PARAM_USERID => {
				let option_value = option.value.as_ref().unwrap();
				if option_value.is_string() {
					user_id_o = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
				}
			},
			_ => {}
		}
	}

	let mut error_message: Option<String> = None;

	let mut user_id = *command.user.id.as_u64();
	if user_id_o.is_none() {
		error!("UserID is undefined.");
	}
	else {
		let user_id_r: Result<u64, ParseIntError> = user_id_o.unwrap().parse::<u64>();
		if let Err(error) = user_id_r {
			error!("user_id coundnt convert u64: {:?}", error);
			error_message = Some("UserIDの記述が正しくありません".to_string());
		}
		else {
			user_id = user_id_r.unwrap();
		}
	}

	if error_message.is_none() {
		let user_mem = command.guild_id.unwrap().member(&ctx.http, user_id).await;
		if let Err(error) = user_mem {
			error!("Error: {}", error);
			error_message = Some(format!("IDが見つからないかその他のエラーです: {}", error));
		}
		else {
			let user_mem = user_mem.unwrap();

			let lsc = STATIC_COMPONENTS.lock().await;
			let locked_db = lsc.get_sql_client();
			let user_data = get_user_data(user_id, locked_db).await;
			std::mem::drop(lsc);
			if let Err(error) = user_data {
				error!("DB Error: {:?}", error);
				error_message = Some(format!("IDが見つからないかその他のエラーです: {:?}", error));
			}
			else {
				let user_data = user_data.unwrap();

				if let Err(error) = command.create_response(&ctx.http, |cir| {
					cir
						.kind(InteractionResponseType::ChannelMessageWithSource)
						.interaction_response_data(|cird| {
							cird
								.create_embed(|cm| {
									cm
										.title("ユーザー情報")
										.field("ID", user_data.uid, true)
										.field("名前", convert::format_discord_username(user_mem.user.name.clone(), user_mem.user.discriminator), true)
										.field("アカウント作成日", utc_to_local_format(&user_mem.user.created_at()), true)
										.field("サーバー入鯖日", utc_to_local_format(&user_mem.joined_at.unwrap_or(command.guild_id.unwrap().created_at())), true)
										.field("アカウントタイプ", (if user_data.glacialeur.is_none() { AccountType::Sub } else { AccountType::Main }).to_string(), true)
										.field("Glacialeur", user_data.glacialeur.unwrap_or("なし".to_string()), true)
										.thumbnail(user_mem.user.avatar_url().unwrap_or("".to_string()))
										.color(user_mem.user.accent_colour.unwrap_or(color::normal_color()))
								})
								.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
						})
				}).await {
					error!("Error: {}", error);
				}

				return;
			}
		}
	}

	if let Err(error) = command.create_interaction_response(&ctx.http, |cir| {
		cir
			.kind(InteractionResponseType::ChannelMessageWithSource)
			.interaction_response_data(|cird| {
				cird
					.create_embed(|cm| {
						cm
							.title("エラー")
							.description(error_message.unwrap())
							.color(color::failed_color())
					})
					.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
			})
	}).await {
		error!("Error: {}", error);
	}
}

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("find")
		.description("ユーザー情報を表示します")
		.kind(ApplicationCommandOptionType::SubCommand);

	for (name, desc, option_type, req) in &PARAMS {
		option
			.create_sub_option(|param_option| {
				param_option
					.name(name)
					.description(desc)
					.kind(*option_type)
					.required(*req)
			});
	}

	return option;
}
