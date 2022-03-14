use log::debug;
use serenity::client::Context;
use serenity::model::interactions::Interaction;
use crate::commands;

pub async fn execute(ctx: Context, interaction: Interaction) {
	debug!("interaction: {:#?}", interaction);
	if let Interaction::ApplicationCommand(command) = interaction {
		commands::interaction_route(ctx, command).await;
	}
	else if let Interaction::MessageComponent(mc) = interaction {
		debug!("\nmcID: {}\nmcType: {:?}\nmcCustomID: {}", mc.id, mc.data.component_type, mc.data.custom_id);
	}
}
