use log::{debug, error};
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption};
//use serenity::model::interactions::InteractionResponseType;
//use serenity::model::interactions::message_component::ButtonStyle;

mod test;
mod version;
mod user;
mod ping;
mod config;

async fn root_commands_route(ctx: Context, command: ApplicationCommandInteraction) {
	if command.data.options.len() != 1 {
		error!("Command option length is not 1.");
		return;
	}

	let sub_command: &ApplicationCommandInteractionDataOption = &command.data.options[0];
	match sub_command.name.as_str() {
		"ping" => ping::execute(ctx, command).await,
		"version" => version::execute(ctx, command).await,
		"config" => config::execute(ctx, command).await,
		"user" => user::commands_route(ctx, &command, sub_command).await,
		_ => error!("No Exist Command!")
	};
}

pub async fn interaction_route(ctx: Context, command: ApplicationCommandInteraction) {
	debug!("\ncommandID: {}\nname: {}", command.id, command.data.name);
	for option in &command.data.options {
		debug!("option name: {}", option.name);
	}

	match command.data.name.as_str() {
		"estella" => root_commands_route(ctx, command).await,
		_ => error!("No Exist Command!")
	}

	/*if let Err(error) = command.create_interaction_response(&ctx.http,
		|res|
			res
				.kind(InteractionResponseType::DeferredChannelMessageWithSource)
	).await {
		error!("{}", error);
	}

	if let Err(error) = command.edit_original_interaction_response(&ctx.http,
		|m|
			m
				.content("Debug OK")
				.components(|c|
					c
						.create_action_row(|ar|
							ar
								.create_button(|b| b.custom_id("123").label("Button1").style(ButtonStyle::Success))
								.create_button(|b| b.custom_id("321").label("Button2").style(ButtonStyle::Danger))
						)
				)
	).await {
		error!("{}", error);
	}*/
}

fn root_command_build(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("estella")
		.description("Estella Command Root")
		.create_option(test::commands_build)
		.create_option(version::command_build)
		.create_option(user::commands_build)
		.create_option(ping::command_build)
		.create_option(config::command_build)
}

pub fn app_commands_build(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
	commands
		.create_application_command(root_command_build)
}
