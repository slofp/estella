use log::error;
use serenity::all::{CommandDataOption, CommandInteraction, CommandOptionType, CreateCommandOption};
use serenity::client::Context;

mod reserve;
mod sub_application;
mod find;

pub async fn commands_route(ctx: Context, command: &CommandInteraction, sub_command: &CommandDataOption) {
	if sub_command.options.len() != 1 {
		error!("Sub command option length is not 1.");
		return;
	}

	let sub_sub_command: &CommandDataOption = &sub_command.options[0];
	match sub_sub_command.name.as_str() {
		"reserve" => reserve::execute(ctx, command, sub_sub_command).await,
		"sub_application" => sub_application::execute(ctx, command, sub_sub_command).await,
		"find" => find::execute(ctx, command, sub_sub_command).await,
		_ => error!("No Exist Command!")
	};
}

pub fn commands_build() -> CreateCommandOption {
	CreateCommandOption::new()
		.name("user")
		.description("Estella User Commands")
		.kind(CommandOptionType::SubCommandGroup)
		.add_sub_option(reserve::command_build)
		.create_sub_option(sub_application::command_build)
		.create_sub_option(find::command_build)
}
