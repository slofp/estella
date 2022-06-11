use std::time::Duration;
use chrono::Utc;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::http::Typing;
use serenity::model::channel::{Embed, EmbedField};
use serenity::model::gateway::Ready;
use serenity::model::interactions::InteractionResponseType;
use serenity::model::interactions::message_component::MessageComponentInteraction;
use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::STATIC_COMPONENTS;
use crate::tables::account;
use crate::tables::quaryfn::{delete_pending_account, get_all_main_pending_account, get_guild_config, get_main_account, get_pending_account, insert_confirmed_account, update_pending_sub_account};
use crate::utils::{color, enums, glacialeur};
use crate::utils::enums::{AccountType, ConfResponseType};

pub async fn execute(ctx: Context, data_about_bot: Ready) {
	for guild in data_about_bot.guilds {
		let guild_id = guild.id();
		debug!("id: {}", guild_id.as_u64() );
		if *guild_id.as_u64() == 0 /* my guild id */ {
			if let Ok(members) = guild_id.members(&ctx.http, None, None).await {
				for member in members {
					if member.user.bot {
						continue;
					}
					let id = glacialeur::generate(
						*member.user.id.as_u64(),
						1 << 3,
						member.joined_at.unwrap_or_else(|| guild_id.created_at()).timestamp() - guild_id.created_at().timestamp()
					);
					println!("{}#{:04}: {}", member.user.name, member.user.discriminator, id);
				}
			}

			break;
		}
	}

	check_vote_task(ctx);
}

pub static ADD_PENDING_USERS: Lazy<Mutex<Vec<account::Pending>>> = Lazy::new(|| Mutex::new(Vec::<account::Pending>::new()));
pub static DEL_PENDING_USERS: Lazy<Mutex<Vec<u64>>> = Lazy::new(|| Mutex::new(Vec::<u64>::new()));

fn check_vote_task(ctx: Context) -> JoinHandle<()> {
	tokio::spawn(async move {
		let lsc = STATIC_COMPONENTS.lock().await;
		let locked_db = lsc.get_sql();
		let mut pending_users = match get_all_main_pending_account(locked_db).await {
			Ok(pv) => pv,
			Err(error) => {
				error!("Error: {:?}", error);
				Vec::<account::Pending>::new()
			}
		};
		std::mem::drop(lsc);
		loop {
			let mut lpu = ADD_PENDING_USERS.lock().await;
			pending_users.append(&mut lpu);
			std::mem::drop(lpu);
			let mut lpu = DEL_PENDING_USERS.lock().await;
			pending_users.retain(|v| !lpu.contains(&v.uid));
			lpu.clear();
			std::mem::drop(lpu);

			let mut del_user_id = Vec::<u64>::new();
			for p_user in &pending_users {
				info!("{}: {} ({:?})", p_user.uid, p_user.name, p_user.end_voting);
				if matches!(p_user.account_type, AccountType::Main) {
					if p_user.end_voting.unwrap() <= Utc::now() {
						end_vote_main_process(&ctx, p_user).await;
						del_user_id.push(p_user.uid);
					}
				}
			}
			pending_users.retain(|v| !del_user_id.contains(&v.uid));

			tokio::time::sleep(Duration::from_secs(1)).await;
		}
	})
}

pub async fn end_conf_sub_process(ctx: &Context, mc: &MessageComponentInteraction, typing_process: Option<Typing>, p_user: &account::Pending, cert_id: u64) {
	info!("End confirmed!");

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let guild_config = get_guild_config(p_user.guild_id, locked_db).await;
	std::mem::drop(lsc);
	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	let guild_config = guild_config.unwrap();

	let message = ctx.http.get_message(guild_config.log_channel_id.unwrap(), p_user.message_id).await;
	if let Err(ref error) = message {
		error!("Error: {:?}", error);
		return;
	}
	let mut message = message.unwrap();

	if let Err(error) = message.edit(&ctx.http, |em| {
		em
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.add_embed(|e| {
				e
					.title("承認完了")
					.description("以下のサブ垢が承認されました！正式に招待可能です")
					.field("ユーザーID", p_user.uid, true)
					.field("名前", &p_user.name, true)
					.color(color::success_color())
			})
	}).await {
		error!("Error: {:?}", error);
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	if let Err(error) = delete_pending_account(p_user, &locked_db).await {
		error!("DB Error: {:?}", error);
	}
	else {
		let confirmed_data =
			if p_user.first_cert.is_some() {
				account::Confirmed {
					uid: p_user.uid,
					name: p_user.name.clone(),
					guild_id: p_user.guild_id,
					account_type: enums::AccountType::Sub,
					main_uid: p_user.main_uid,
					first_cert: p_user.first_cert,
					second_cert: Some(cert_id)
				}
			}
			else {
				account::Confirmed {
					uid: p_user.uid,
					name: p_user.name.clone(),
					guild_id: p_user.guild_id,
					account_type: enums::AccountType::Sub,
					main_uid: p_user.main_uid,
					first_cert: Some(cert_id),
					second_cert: None
				}
			};
		if let Err(error) = insert_confirmed_account(&confirmed_data, locked_db).await {
			error!("DB Error: {:?}", error);
		}
	}
	std::mem::drop(lsc);

	if let Some(typing_process) = typing_process {
		typing_process.stop();
	}
	conf_result_send_message(ctx, mc, ConfResponseType::Success, "").await;
}

pub async fn end_vote_main_process(ctx: &Context, p_user: &account::Pending) {
	info!("End vote!");

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let guild_config = get_guild_config(p_user.guild_id, locked_db).await;
	std::mem::drop(lsc);
	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	let guild_config = guild_config.unwrap();

	let message = ctx.http.get_message(guild_config.log_channel_id.unwrap(), p_user.message_id).await;
	if let Err(ref error) = message {
		error!("Error: {:?}", error);
		return;
	}
	let mut message = message.unwrap();

	if let Err(error) = message.edit(&ctx.http, |em| {
		em
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.add_embed(|e| {
				e
					.title("投票終了")
					.description("以下の内容を正式に登録されました！正式に招待可能です")
					.field("ユーザーID", p_user.uid, true)
					.field("名前", &p_user.name, true)
					.color(color::success_color())
			})
	}).await {
		error!("Error: {:?}", error);
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	if let Err(error) = delete_pending_account(p_user, &locked_db).await {
		error!("DB Error: {:?}", error);
	}
	else {
		let confirmed_data = account::Confirmed {
			uid: p_user.uid,
			name: p_user.name.clone(),
			guild_id: p_user.guild_id,
			account_type: enums::AccountType::Main,
			main_uid: None,
			first_cert: None,
			second_cert: None
		};
		if let Err(error) = insert_confirmed_account(&confirmed_data, locked_db).await {
			error!("DB Error: {:?}", error);
		}
	}
	std::mem::drop(lsc);
}

pub async fn reject_vote_process(ctx: &Context, guild_id: u64, user_id: u64) {
	info!("Reject vote...");
	let mut lpu = DEL_PENDING_USERS.lock().await;
	lpu.push(user_id);
	std::mem::drop(lpu);

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let guild_config  = get_guild_config(guild_id, &locked_db).await;
	let p_user = get_pending_account(guild_id, user_id, &locked_db).await;
	std::mem::drop(lsc);

	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	if let Err(error) = p_user {
		error!("DB Error: {:?}", error);
		return;
	}
	let guild_config = guild_config.unwrap();
	let p_user = p_user.unwrap();

	let message = ctx.http.get_message(guild_config.log_channel_id.unwrap(), p_user.message_id).await;
	if let Err(ref error) = message {
		error!("Error: {:?}", error);
		return;
	}
	let mut message = message.unwrap();

	if let Err(error) = message.edit(&ctx.http, |em| {
		em
			.components(|c| {
				c.set_action_rows(vec![])
			})
			.set_embeds(vec![])
			.add_embed(|e| {
				e
					.title("申請却下")
					.description("以下の申請を取り下げました")
					.field("ユーザーID", p_user.uid, true)
					.field("名前", &p_user.name, true)
					.color(color::critical_color())
			})
	}).await {
		error!("Error: {:?}", error);
	}

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	if let Err(error) = delete_pending_account(&p_user, &locked_db).await {
		error!("DB Error: {:?}", error);
	}
	std::mem::drop(lsc);
}

pub async fn conf_process(ctx: &Context, mc: &MessageComponentInteraction, guild_id: u64, user_id: u64, conf_id: u64) {
	info!("confirm...");

	if let Err(error) = mc.defer(&ctx.http).await {
		error!("{}", error);
	}

	let typing_process = match mc.channel_id.start_typing(&ctx.http) {
		Ok(v) => Some(v),
		Err(_) => None
	};

	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	let guild_config  = get_guild_config(guild_id, &locked_db).await;
	let p_user = get_pending_account(guild_id, user_id, &locked_db).await;
	let c_user = get_main_account(guild_id, conf_id, &locked_db).await;
	std::mem::drop(lsc);

	if let Err(error) = guild_config {
		error!("DB Error: {:?}", error);
		return;
	}
	if let Err(error) = p_user {
		error!("DB Error: {:?}", error);
		return;
	}
	if let Err(error) = c_user {
		error!("DB Error: {:?}", error);
		return;
	}
	let guild_config = guild_config.unwrap();
	let mut p_user = p_user.unwrap();
	let c_user = c_user.unwrap();

	if c_user.is_leaved {
		return;
	}
	if p_user.first_cert.is_some() || c_user.is_sc {
		if p_user.first_cert.unwrap_or_else(|| 0) != conf_id {
			end_conf_sub_process(&ctx, &mc, typing_process, &p_user, conf_id).await;
		}
		else {
			if let Some(typing_process) = typing_process {
				typing_process.stop();
			}
			conf_result_send_message(ctx, mc, ConfResponseType::ExistErr, "").await;
		}
		return;
	}

	let message = ctx.http.get_message(guild_config.log_channel_id.unwrap(), p_user.message_id).await;
	if let Err(ref error) = message {
		error!("Error: {:?}", error);
		conf_result_send_message(ctx, mc, ConfResponseType::OtherErr, "メッセージが見つかりません").await;
		return;
	}
	let mut message = message.unwrap();
	if message.embeds.len() == 0 {
		error!("Error: Not found embed");
		conf_result_send_message(ctx, mc, ConfResponseType::OtherErr, "メッセージが見つかりません").await;
		return;
	}
	let mut message_embed: Embed = message.embeds[0].clone();
	message_embed.fields.push(EmbedField::new("第一承認者ID", conf_id.to_string(), true));

	if let Err(error) = message.edit(&ctx.http, |em| {
		em
			.set_embeds(vec![ CreateEmbed::from(message_embed) ])
	}).await {
		error!("Error: {:?}", error);
	}

	p_user.first_cert = Some(conf_id);
	let lsc = STATIC_COMPONENTS.lock().await;
	let locked_db = lsc.get_sql();
	if let Err(error) = update_pending_sub_account(&p_user, &locked_db).await {
		error!("DB Error: {:?}", error);
		conf_result_send_message(ctx, mc, ConfResponseType::OtherErr, error).await;
		return;
	}
	std::mem::drop(lsc);

	if let Some(typing_process) = typing_process {
		typing_process.stop();
	}
	conf_result_send_message(ctx, mc, ConfResponseType::Ok, "").await;
}

pub async fn conf_result_send_message<E>(ctx: &Context, mc: &MessageComponentInteraction, cr_type: enums::ConfResponseType, error: E) where E: std::fmt::Debug {
	match cr_type {
		ConfResponseType::Ok => {
			if let Err(error) = mc.create_followup_message(&ctx.http, |irf| {
				irf
					.create_embed(|e| {
						e
							.title("完了")
							.description("承認しました！残り1人の承認が必要になります")
							.color(color::success_color())
					})
					.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
			}).await {
				error!("Error: {:?}", error);
			}
		}
		ConfResponseType::EqualErr => {
			if let Err(error) = mc.create_interaction_response(&ctx.http, |ir| {
				ir
					.interaction_response_data(|ird| {
						ird
							.create_embed(|e| {
								e
									.title("エラー")
									.description("登録者は承認できません")
									.color(color::failed_color())
							})
							.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
					})
					.kind(InteractionResponseType::ChannelMessageWithSource)
			}).await {
				error!("Error: {:?}", error);
			}
		}
		ConfResponseType::ExistErr => {
			if let Err(error) = mc.create_followup_message(&ctx.http, |irf| {
				irf
					.create_embed(|e| {
						e
							.title("エラー")
							.description("すでに承認されています")
							.color(color::failed_color())
					})
					.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
			}).await {
				error!("Error: {:?}", error);
			}
		}
		ConfResponseType::OtherErr => {
			if let Err(error) = mc.create_followup_message(&ctx.http, |irf| {
				irf
					.create_embed(|e| {
						e
							.title("エラー")
							.description(format!("{:?}", error))
							.color(color::failed_color())
					})
					.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
			}).await {
				error!("Error: {:?}", error);
			}
		}
		ConfResponseType::Success => {
			if let Err(error) = mc.create_followup_message(&ctx.http, |irf| {
				irf
					.create_embed(|e| {
						e
							.title("完了")
							.description("承認しました！")
							.color(color::success_color())
					})
					.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
			}).await {
				error!("Error: {:?}", error);
			}
		}
	}
}
