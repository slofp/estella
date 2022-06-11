use std::num::ParseIntError;
use chrono::{Duration, Local, Utc};
use log::error;
use serenity::builder::{CreateApplicationCommandOption, CreateComponents};
use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::{ApplicationCommandInteractionDataOption, ApplicationCommandInteraction, ApplicationCommandOptionType};
use serenity::model::interactions::{InteractionApplicationCommandCallbackDataFlags, InteractionResponseType};
use serenity::model::interactions::message_component::ButtonStyle;
use crate::events::ready_event::ADD_PENDING_USERS;
use crate::STATIC_COMPONENTS;
use crate::tables::{account, quaryfn};
use crate::utils::color;
use crate::utils::enums::AccountType;

const PARAM_USERID: &str = "user_id";
const PARAM_NAME: &str = "name";
const PARAM_REASON: &str = "reason";

/*
Paramsは値名→説明→型定義で構成されています
*/
const PARAMS: [(&str, &str, ApplicationCommandOptionType); 3] = [
	(PARAM_USERID, "ユーザーID", ApplicationCommandOptionType::String),
	(PARAM_NAME, "登録名", ApplicationCommandOptionType::String),
	(PARAM_REASON, "登録理由", ApplicationCommandOptionType::String),
];

pub async fn execute(ctx: Context, command: &ApplicationCommandInteraction, command_args: &ApplicationCommandInteractionDataOption) {
	let mut user_id: Option<String> = None;
	let mut name: Option<String> = None;
	let mut reason: Option<String> = None;

	for option in &command_args.options {
		//info!("option data: {} [{:?}]", b.name, b.value);
		match (&option.name).as_str() {
			PARAM_USERID => {
				let option_value = option.value.as_ref().unwrap();
				if option_value.is_string() {
					user_id = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
				}
			},
			PARAM_NAME => {
				let option_value = option.value.as_ref().unwrap();
				if option_value.is_string() {
					name = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
				}
			},
			PARAM_REASON => {
				let option_value = option.value.as_ref().unwrap();
				if option_value.is_string() {
					reason = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
				}
			},
			_ => {}
		}
	}

	let mut error_message: Option<String> = None;

	if user_id.is_none() {
		error!("UserID is undefined.");
		error_message = Some("UserIDが入力されていません".to_string());
	}
	else if name.is_none() {
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
		if let Err(error) = command.create_interaction_response(&ctx.http,
			|res|
				res
					.kind(InteractionResponseType::ChannelMessageWithSource)
					.interaction_response_data(|m| {
						m
							.create_embed(|e| {
								e
									.title("エラー")
									.description(err_msg_f)
									.color(color::failed_color())
							})
					})
		).await {
			error!("{}", error);
		}
		return;
	}

	let user_id: u64 = user_id_r.unwrap().unwrap();
	let name: String = name.unwrap();

	if let Err(error) = command.create_interaction_response(&ctx.http,
		|res|
			res
				.kind(InteractionResponseType::ChannelMessageWithSource)
				.interaction_response_data(|m| {
					m
						.create_embed(|e| {
							e
								.title("確認")
								.description("以下の内容で登録します")
								.field("ユーザーID", &user_id, true)
								.field("名前", &name, true)
								.color(color::normal_color())
						})
						.components(|com| {
							com.create_action_row(|ar| {
								ar
									.create_button(|b| {
										b.custom_id(format!("ok_{}", &user_id)).style(ButtonStyle::Success).label("OK")
									})
									.create_button(|b| {
										b.custom_id(format!("cancel_{}", &user_id)).style(ButtonStyle::Danger).label("キャンセル")
									})
							})
						})
						.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
				})
	).await {
		error!("{}", error);
		return;
	}

	let message = command.get_interaction_response(&ctx.http).await.unwrap();
	let button_interaction =
		match message.await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("ok_{}", &user_id) {
		/*if let Err(error) = message.delete(&ctx.http).await {
			error!("{}", error);
			return;
		}*/
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}
		if let Err(error) = button_interaction.edit_original_interaction_response(&ctx.http, |res| {
			res
				.components(|c| {
					c.set_action_rows(vec![])
				})
				.set_embeds(vec![])
				.create_embed(|e| {
					e
						.title("処理中...")
						.description("このメッセージを削除しないでください")
						.color(color::normal_color())
				})
		}).await {
			error!("{}", error);
			return;
		}

		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if command.guild_id.is_none() {
			error!("Should not occur error: GuildID is none");
			return;
		}
		let guild_config = match quaryfn::get_guild_config(*command.guild_id.unwrap().as_u64(), &locked_db).await {
			Ok(x) => Some(x),
			Err(e) => {
				error!("DB Error: {:?}", e);
				error_message = Some(format!("{:?}", e));
				None
			}
		};
		if let Some(guild_config) = guild_config {
			if let Some(guild_log_channel) = guild_config.log_channel_id {
				let end_vote_time = Utc::now() + Duration::days(7);
				let log_channel = ChannelId(guild_log_channel);
				let vote_message = log_channel.send_message(&ctx.http, |m| {
					m
						.add_embed(|e| {
							e
								.title("追加申請")
								.description("以下の内容で登録申請されました。内容を見てこのサーバーに入れたくないと判断した場合は「却下」ボタンを押してください")
								.field("ユーザーID", &user_id, true)
								.field("名前", &name, true)
								.field("申請却下終了時刻", &end_vote_time.with_timezone(&Local).format("%Y/%m/%d %H:%M:%S"), true)
								.color(color::normal_color());
							if let Some(reason) = reason {
								e
									.field("申請理由", reason, true);
							}
							e
						})
						.components(|com| {
							com.create_action_row(|ar| {
								ar
									.create_button(|b| {
										b.custom_id(format!("reject_{}", &user_id)).style(ButtonStyle::Danger).label("申請を却下する")
									})
							})
						})
				}).await;
				if let Err(ref error) = vote_message {
					error!("Error: {:?}", error);
					return;
				}
				let vote_message = vote_message.unwrap();

				let pending_data = account::Pending {
					uid: user_id,
					name: name.clone(),
					message_id: *(&vote_message).id.as_u64(),
					end_voting: Some(Utc::now() + Duration::seconds(30)), // + Duration::days(7),
					guild_id: guild_config.uid,
					account_type: AccountType::Main,
					main_uid: None,
					first_cert: None
				};

				if let Err(error) = quaryfn::insert_pending_account(&pending_data, &locked_db).await {
					error!("DB Error: {:?}", error);
					error_message = Some(format!("{:?}", error));
				}
				else {
					let mut lpu = ADD_PENDING_USERS.lock().await;
					lpu.push(pending_data);
					std::mem::drop(lpu);
				}
			}
			else {
				error!("Error: Not found log channel");
				error_message = Some(String::from("ログチャンネルが指定されていません"));
			}
		}

		std::mem::drop(lsc);

		if let Err(error) = button_interaction.edit_original_interaction_response(&ctx.http, |res| {
			res
				.components(|c| {
					c.set_action_rows(vec![])
				})
				.set_embeds(vec![])
				.create_embed(|e| {
					if let Some(err_msg) = error_message {
						e
							.title("エラー")
							.description(err_msg)
							.color(color::failed_color())
					}
					else {
						e
							.title("完了")
							.description("以下の内容で登録しました！7日間何も無ければ正式に登録されます")
							.field("ユーザーID", &user_id, true)
							.field("名前", &name, true)
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}
		//button_interaction.defer(&ctx.http).await;
	}
	else if button_interaction.data.custom_id == format!("cancel_{}", &user_id) {
		/*if let Err(error) = message.delete(&ctx.http).await {
			error!("{}", error);
			return;
		}*/
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.set_components(CreateComponents::default())
						.embeds([])
						.create_embed(|e| {
							e
								.title("キャンセル")
								.description("処理を取り消しました")
								.color(color::normal_color())
						})
				})
		}).await {
			error!("{}", error);
			return;
		}
		//button_interaction.defer(&ctx.http).await;
	}
}

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("reserve")
		.description("ユーザー登録を予約します")
		.kind(ApplicationCommandOptionType::SubCommand);

	for (name, desc, option_type) in &PARAMS {
		option
			.create_sub_option(|param_option| {
				param_option
					.name(name)
					.description(desc)
					.kind(*option_type)
			});
	}

	return option;
}
