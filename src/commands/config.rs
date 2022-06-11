use std::sync::Arc;
use log::{error, info};
use serenity::builder::CreateApplicationCommandOption;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType};
use serenity::model::interactions::InteractionResponseType;
use serenity::model::interactions::message_component::{ButtonStyle, MessageComponentInteraction};
use crate::STATIC_COMPONENTS;
use crate::tables::quaryfn::{update_guild_config_auth, update_guild_config_bot, update_guild_config_leave, update_guild_config_log, update_guild_config_white};
use crate::utils::color;

const MENU_PARAM_LC: &str = "log_channel";
const MENU_PARAM_AR: &str = "auth_role";
const MENU_PARAM_BR: &str = "bot_role";
const MENU_PARAM_WL: &str = "white_list";
const MENU_PARAM_LB: &str = "leave_ban";

pub async fn execute(ctx: Context, command: ApplicationCommandInteraction) {
	if let Err(error) = command.create_interaction_response(&ctx.http,
		|res|
			res
				.kind(InteractionResponseType::ChannelMessageWithSource)
				.interaction_response_data(|ird| {
					ird
						.components(|cc| {
							cc
								.create_action_row(|car| {
									car
										.create_select_menu(|csm| {
											csm
												.custom_id(format!("c-config_{}", command.user.id.as_u64()))
												.placeholder("設定したい項目を選択してください")
												.options(|csmos| {
													csmos
														.create_option(|csmo| {
															csmo
																.label("ログチャンネル設定")
																.description("ログチャンネルの設定をします")
																.value(MENU_PARAM_LC)
														})
														.create_option(|csmo| {
															csmo
																.label("認証ロール設定")
																.description("認証ロールの設定をします")
																.value(MENU_PARAM_AR)
														})
														.create_option(|csmo| {
															csmo
																.label("Botロール設定")
																.description("Botロールの設定をします")
																.value(MENU_PARAM_BR)
														})
														.create_option(|csmo| {
															csmo
																.label("ホワイトリスト設定")
																.description("ホワイトリスト設定をします")
																.value(MENU_PARAM_WL)
														})
														.create_option(|csmo| {
															csmo
																.label("退鯖BAN設定")
																.description("サーバーを抜けたときの設定をします")
																.value(MENU_PARAM_LB)
														})
												})
												.min_values(1)
												.max_values(1)
										})
								})
						})
						.create_embed(|cm| {
							cm
								.title("設定")
								.description("下の選択メニューから設定したい項目を選択してください。")
								.color(color::normal_color())
						})
				})
	).await {
		error!("{}", error);
		return;
	}

	let message = command.get_interaction_response(&ctx.http).await.unwrap();
	let select_interaction =
		match message.await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if let Err(error) = select_interaction.defer(&ctx.http).await {
		error!("{}", error);
		return;
	}
	for menu_value in &select_interaction.data.values {
		match menu_value.as_str() {
			MENU_PARAM_LC => log_channel_config(&ctx, &command, select_interaction.clone()).await,
			MENU_PARAM_AR => auth_role_config(&ctx, &command, select_interaction.clone()).await,
			MENU_PARAM_BR => bot_role_config(&ctx, &command, select_interaction.clone()).await,
			MENU_PARAM_WL => white_list_config(&ctx, &command, select_interaction.clone()).await,
			MENU_PARAM_LB => leave_ban_config(&ctx, &command, select_interaction.clone()).await,
			_ => {}
		}
	}
}

async fn log_channel_config(ctx: &Context, command: &ApplicationCommandInteraction, select_interaction: Arc<MessageComponentInteraction>) {
	let edit_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("ログチャンネル設定")
					.description("ログチャンネルにするチャンネルのIDを入力してください。")
					.color(color::normal_color())
			})
	}).await;
	if let Err(error) = edit_message {
		error!("Error: {}", error);
		return;
	}
	let edit_message = edit_message.unwrap();

	let rep_message = match edit_message.channel_id.await_reply(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
		None => {
			error!("Wait message timeout...");
			return;
		}
		Some(x) => x
	};

	if let Err(error) = rep_message.delete(&ctx.http).await {
		error!("{:?}", error);
	}

	let mut error_message: Option<String> = None;
	let mut channel_id_r = rep_message.content.clone().parse::<u64>();
	if let Err(ref error) = channel_id_r {
		error!("channel_id coundnt convert u64: {:?}", error);
		error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
	}
	if let Some(error_message) = error_message {
		if let Err(error) = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
			eir
				.components(|c| {
					c.set_action_rows(vec![])
				})
				.set_embeds(vec![])
				.create_embed(|e| {
					e
						.title("エラー")
						.description(error_message)
						.color(color::failed_color())
				})
		}).await {
			error!("Error: {}", error);
		}
		return;
	}
	let channel_id: u64 = channel_id_r.unwrap();

	let confirm_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("確認")
					.description("以下の内容で設定します")
					.field("チャンネルID", &channel_id, true)
					.color(color::normal_color())
			})
			.components(|com| {
				com
					.set_action_rows(vec![])
					.create_action_row(|ar| {
						ar
							.create_button(|b| {
								b.custom_id(format!("ok_{}", &channel_id)).style(ButtonStyle::Success).label("OK")
							})
							.create_button(|b| {
								b.custom_id(format!("cancel_{}", &channel_id)).style(ButtonStyle::Danger).label("キャンセル")
							})
					})
			})
	}).await;
	if let Err(error) = confirm_message {
		error!("Error: {}", error);
		return;
	}

	let button_interaction =
		match confirm_message.unwrap().await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("ok_{}", &channel_id) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		error_message = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_log(*command.guild_id.unwrap().as_u64(), channel_id, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("以下の内容で設定しました！")
							.field("チャンネルID", &channel_id, true)
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}

	}
	else {
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.components(|c| {
							c.set_action_rows(vec![])
						})
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
	}
}

async fn auth_role_config(ctx: &Context, command: &ApplicationCommandInteraction, select_interaction: Arc<MessageComponentInteraction>) {
	let edit_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("認証ロール設定")
					.description("承認したときに使用するロールのIDを入力してください。")
					.color(color::normal_color())
			})
	}).await;
	if let Err(error) = edit_message {
		error!("Error: {}", error);
		return;
	}
	let edit_message = edit_message.unwrap();

	let rep_message = match edit_message.channel_id.await_reply(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
		None => {
			error!("Wait message timeout...");
			return;
		}
		Some(x) => x
	};

	if let Err(error) = rep_message.delete(&ctx.http).await {
		error!("{:?}", error);
	}

	let mut error_message: Option<String> = None;
	let mut auth_id_r = rep_message.content.clone().parse::<u64>();
	if let Err(ref error) = auth_id_r {
		error!("auth_id coundnt convert u64: {:?}", error);
		error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
	}
	if let Some(error_message) = error_message {
		if let Err(error) = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
			eir
				.components(|c| {
					c.set_action_rows(vec![])
				})
				.set_embeds(vec![])
				.create_embed(|e| {
					e
						.title("エラー")
						.description(error_message)
						.color(color::failed_color())
				})
		}).await {
			error!("Error: {}", error);
		}
		return;
	}
	let auth_id: u64 = auth_id_r.unwrap();

	let confirm_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("確認")
					.description("以下の内容で設定します")
					.field("ロールID", &auth_id, true)
					.color(color::normal_color())
			})
			.components(|com| {
				com
					.set_action_rows(vec![])
					.create_action_row(|ar| {
						ar
							.create_button(|b| {
								b.custom_id(format!("ok_{}", &auth_id)).style(ButtonStyle::Success).label("OK")
							})
							.create_button(|b| {
								b.custom_id(format!("cancel_{}", &auth_id)).style(ButtonStyle::Danger).label("キャンセル")
							})
					})
			})
	}).await;
	if let Err(error) = confirm_message {
		error!("Error: {}", error);
		return;
	}

	let button_interaction =
		match confirm_message.unwrap().await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("ok_{}", &auth_id) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		error_message = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_auth(*command.guild_id.unwrap().as_u64(), auth_id, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("以下の内容で設定しました！")
							.field("ロールID", &auth_id, true)
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}

	}
	else {
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.components(|c| {
							c.set_action_rows(vec![])
						})
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
	}
}

async fn bot_role_config(ctx: &Context, command: &ApplicationCommandInteraction, select_interaction: Arc<MessageComponentInteraction>) {
	let edit_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("Botロール設定")
					.description("Botがサーバーに入った時に使用するロールのIDを入力してください。")
					.color(color::normal_color())
			})
	}).await;
	if let Err(error) = edit_message {
		error!("Error: {}", error);
		return;
	}
	let edit_message = edit_message.unwrap();

	let rep_message = match edit_message.channel_id.await_reply(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
		None => {
			error!("Wait message timeout...");
			return;
		}
		Some(x) => x
	};

	if let Err(error) = rep_message.delete(&ctx.http).await {
		error!("{:?}", error);
	}

	let mut error_message: Option<String> = None;
	let mut bot_id_r = rep_message.content.clone().parse::<u64>();
	if let Err(ref error) = bot_id_r {
		error!("auth_id coundnt convert u64: {:?}", error);
		error_message = Some(format!("チャンネルIDの記述が正しくありません: {:?}", error).to_string());
	}
	if let Some(error_message) = error_message {
		if let Err(error) = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
			eir
				.components(|c| {
					c.set_action_rows(vec![])
				})
				.set_embeds(vec![])
				.create_embed(|e| {
					e
						.title("エラー")
						.description(error_message)
						.color(color::failed_color())
				})
		}).await {
			error!("Error: {}", error);
		}
		return;
	}
	let bot_id: u64 = bot_id_r.unwrap();

	let confirm_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("確認")
					.description("以下の内容で設定します")
					.field("ロールID", &bot_id, true)
					.color(color::normal_color())
			})
			.components(|com| {
				com
					.set_action_rows(vec![])
					.create_action_row(|ar| {
						ar
							.create_button(|b| {
								b.custom_id(format!("ok_{}", &bot_id)).style(ButtonStyle::Success).label("OK")
							})
							.create_button(|b| {
								b.custom_id(format!("cancel_{}", &bot_id)).style(ButtonStyle::Danger).label("キャンセル")
							})
					})
			})
	}).await;
	if let Err(error) = confirm_message {
		error!("Error: {}", error);
		return;
	}

	let button_interaction =
		match confirm_message.unwrap().await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("ok_{}", &bot_id) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		error_message = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_bot(*command.guild_id.unwrap().as_u64(), bot_id, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("以下の内容で設定しました！")
							.field("ロールID", &bot_id, true)
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}

	}
	else {
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.components(|c| {
							c.set_action_rows(vec![])
						})
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
	}
}

async fn white_list_config(ctx: &Context, command: &ApplicationCommandInteraction, select_interaction: Arc<MessageComponentInteraction>) {
	let confirm_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("ホワイトリスト設定")
					.description("このサーバーをホワイトリスト制御しますか？")
					.color(color::normal_color())
			})
			.components(|com| {
				com
					.set_action_rows(vec![])
					.create_action_row(|ar| {
						ar
							.create_button(|b| {
								b.custom_id(format!("yes_{}", command.user.id.as_u64())).style(ButtonStyle::Success).label("はい")
							})
							.create_button(|b| {
								b.custom_id(format!("no_{}", command.user.id.as_u64())).style(ButtonStyle::Danger).label("いいえ")
							})
							.create_button(|b| {
								b.custom_id(format!("cancel_{}", command.user.id.as_u64())).style(ButtonStyle::Secondary).label("キャンセル")
							})
					})
			})
	}).await;
	if let Err(error) = confirm_message {
		error!("Error: {}", error);
		return;
	}

	let button_interaction =
		match confirm_message.unwrap().await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("yes_{}", command.user.id.as_u64()) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		let mut error_message: Option<String> = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_white(*command.guild_id.unwrap().as_u64(), true, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("ホワイトリスト制御を有効にしました！")
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}
	}
	else if button_interaction.data.custom_id == format!("no_{}", command.user.id.as_u64()) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		let mut error_message: Option<String> = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_white(*command.guild_id.unwrap().as_u64(), false, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("ホワイトリスト制御を無効にしました！")
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}
	}
	else {
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.components(|c| {
							c.set_action_rows(vec![])
						})
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
	}
}

async fn leave_ban_config(ctx: &Context, command: &ApplicationCommandInteraction, select_interaction: Arc<MessageComponentInteraction>) {
	let confirm_message = select_interaction.edit_original_interaction_response(&ctx.http, |eir| {
		eir
			.set_embeds(vec![])
			.create_embed(|e| {
				e
					.title("退出時BAN設定")
					.description("このサーバーの退出時BANを有効にしますか？")
					.color(color::normal_color())
			})
			.components(|com| {
				com
					.set_action_rows(vec![])
					.create_action_row(|ar| {
						ar
							.create_button(|b| {
								b.custom_id(format!("yes_{}", command.user.id.as_u64())).style(ButtonStyle::Success).label("はい")
							})
							.create_button(|b| {
								b.custom_id(format!("no_{}", command.user.id.as_u64())).style(ButtonStyle::Danger).label("いいえ")
							})
							.create_button(|b| {
								b.custom_id(format!("cancel_{}", command.user.id.as_u64())).style(ButtonStyle::Secondary).label("キャンセル")
							})
					})
			})
	}).await;
	if let Err(error) = confirm_message {
		error!("Error: {}", error);
		return;
	}

	let button_interaction =
		match confirm_message.unwrap().await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60 * 3)).await {
			Some(x) => x,
			None => {
				error!("interaction timeout...");
				return;
			}
		};

	if button_interaction.data.custom_id == format!("yes_{}", command.user.id.as_u64()) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		let mut error_message: Option<String> = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_leave(*command.guild_id.unwrap().as_u64(), true, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("退出時BANを有効にしました！")
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}
	}
	else if button_interaction.data.custom_id == format!("no_{}", command.user.id.as_u64()) {
		if let Err(error) = button_interaction.defer(&ctx.http).await {
			error!("{}", error);
			return;
		}

		let mut error_message: Option<String> = None;
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		if let Err(error) = update_guild_config_leave(*command.guild_id.unwrap().as_u64(), false, locked_db).await {
			error!("{:?}", error);
			error_message = Some(format!("{:?}", error));
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
							.description("退出時BANを無効にしました！")
							.color(color::success_color())
					}
				})
		}).await {
			error!("{}", error);
			return;
		}
	}
	else {
		if let Err(error) = button_interaction.create_interaction_response(&ctx.http, |res| {
			res
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|m| {
					m
						.components(|c| {
							c.set_action_rows(vec![])
						})
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
	}
}

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("config")
		.description("Botの設定をします")
		.kind(ApplicationCommandOptionType::SubCommand)
}
