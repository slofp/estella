use log::error;
use serenity::builder::CreateApplicationCommandOption;
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType};

mod create;
mod remove;

pub async fn commands_route(ctx: Context, command: &ApplicationCommandInteraction, sub_command: &ApplicationCommandInteractionDataOption) {
	if sub_command.options.len() != 1 {
		error!("Sub command option length is not 1.");
		return;
	}

	let sub_sub_command: &ApplicationCommandInteractionDataOption = &sub_command.options[0];
	match sub_sub_command.name.as_str() {
		"create" => create::execute(ctx, command, sub_sub_command).await,
		"remove" => remove::execute(ctx, command, sub_sub_command).await,
		_ => error!("No Exist Command!")
	};
}

pub fn commands_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("mails")
		.description("Estella Mails Commands")
		.kind(ApplicationCommandOptionType::SubCommandGroup)
		.create_sub_option(create::command_build)
		.create_sub_option(remove::command_build)
}

