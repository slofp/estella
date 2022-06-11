use log::{debug, info};
use serenity::client::Context;
use serenity::model::interactions::Interaction;
use crate::commands;
use crate::events::ready_event::{conf_process, conf_result_send_message, reject_vote_process};
use crate::utils::enums::ConfResponseType;

pub async fn execute(ctx: Context, interaction: Interaction) {
	debug!("interaction: {:#?}", interaction);
	if let Interaction::ApplicationCommand(command) = interaction {
		commands::interaction_route(ctx, command).await;
	}
	else if let Interaction::MessageComponent(mc) = interaction {
		debug!("\nmcID: {}\nmcType: {:?}\nmcCustomID: {}", mc.id, mc.data.component_type, mc.data.custom_id);
		if mc.data.custom_id.starts_with("reject") {
			let clone_custom_id = mc.data.custom_id.clone();
			let split_custom_id: Vec<&str> = clone_custom_id.split("_").collect();
			let user_id = split_custom_id[1];
			let user_id: u64 = user_id.parse::<u64>().unwrap();
			info!("{}", user_id);
			reject_vote_process(&ctx, *mc.guild_id.unwrap().as_u64(), user_id).await;
		}
		else if mc.data.custom_id.starts_with("conf") {
			let clone_custom_id = mc.data.custom_id.clone();
			let split_custom_id: Vec<&str> = clone_custom_id.split("_").collect();
			let user_id = split_custom_id[1];
			let user_id: u64 = user_id.parse::<u64>().unwrap();
			info!("{}", user_id);
			info!("{}", *mc.user.id.as_u64());
			if *mc.user.id.as_u64() == user_id {
				info!("user_id is equal pressed button user id");
				conf_result_send_message(&ctx, &mc, ConfResponseType::EqualErr, "").await;
				return;
			}
			let p_user_id = split_custom_id[2];
			let p_user_id: u64 = p_user_id.parse::<u64>().unwrap();
			info!("{}", p_user_id);
			conf_process(&ctx, &mc, *mc.guild_id.unwrap().as_u64(), p_user_id, *mc.user.id.as_u64()).await;
		}
	}
}
