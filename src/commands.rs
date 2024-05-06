use std::convert::Into;
use log::{debug, error};
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use crate::command_define::{BaseCommand, CommonCommandType};
use crate::commands::ping::PingCommand;
//use serenity::model::interactions::InteractionResponseType;
//use serenity::model::interactions::message_component::ButtonStyle;

mod version;
mod user;
mod ping;
mod config;

static COMMANDSS: Vec<CommonCommandType> = vec![
	PingCommand::new().into()
];

async fn root_commands_route(ctx: Context, command: CommandInteraction) -> serenity::Result<()> {
	if command.data.options.len() != 1 {
		error!("Command option length is not 1.");
		return Ok(());
	}

	let mut executed = false;
	let sub_command: &CommandDataOption = &command.data.options[0];
	let sub_command_kind = sub_command.kind();
	let sub_command_name = sub_command.name.to_string();
	if matches!(sub_command_kind, CommandOptionType::SubCommand) {
		for sub_cmd in COMMANDSS {
			if matches!(sub_cmd, CommonCommandType::SubCommand(_)) {
				continue;
			}
			if let CommonCommandType::Command(cmd) = sub_cmd {
				if cmd.get_name() == sub_command_name {
					let CommandDataOptionValue::SubCommand(sub_command_value) = sub_command.to_owned().value;
					cmd.execute(ctx, command, sub_command_value).await?;
					executed = true;
					break;
				}
			}
		}
	}
	else if matches!(sub_command_kind, CommandOptionType::SubCommandGroup) {
		for sub_cmd in COMMANDSS {
			if matches!(sub_cmd, CommonCommandType::Command(_)) {
				continue;
			}
			if let CommonCommandType::SubCommand(cmd) = sub_cmd {
				if cmd.get_name() == sub_command_name {
					let CommandDataOptionValue::SubCommandGroup(sub_command_value) = sub_command.to_owned().value;
					cmd.commands_route(ctx, command, sub_command_value[0].to_owned()).await?;
					executed = true;
					break;
				}
			}
		}
	}

	if !executed {
		error!("No Exist Command!");
	}

	Ok(())

	/* old
		match .as_str() {
		"ping" => ping::execute(ctx, command).await,
		"version" => version::execute(ctx, command).await,
		"config" => config::execute(ctx, command).await,
		"user" => user::commands_route(ctx, &command, sub_command).await,
		_ =>
	};
	*/
}

pub async fn interaction_route(ctx: Context, command: CommandInteraction) {
	debug!("\ncommandID: {}\nname: {}", command.id, command.data.name);
	for option in &command.data.options {
		debug!("option name: {}", option.name);
	}

	if command.user.bot {
		return;
	}

	let res = match command.data.name.as_str() {
		"estella" => root_commands_route(ctx, command).await,
		_ => {
			error!("No Exist Command!");
			Ok(())
		}
	};
	if let Err(error) = res {
		error!("{}", error);
	}

	/* edit時のボタン追加コード？
	if let Err(error) = command.create_interaction_response(&ctx.http,
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

fn root_command_build(command: CreateCommand) -> CreateCommand {
	command
		.name("estella")
		.description("Estella Command Root")
		.add_option(version::command_build())
		.add_option(user::commands_build())
		.add_option(ping::command_build())
		.add_option(config::command_build())
		.add_option(mails::commands_build())
}

pub fn app_commands_build() -> CreateCommand {
	root_command_build(CreateCommand::new(""))
}
