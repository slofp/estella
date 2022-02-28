use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};

pub mod test;
pub mod ver;
pub mod user;

fn root_command_build(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("estella")
		.description("Estella Command Root")
		.create_option(test::commands_build)
		.create_option(ver::command_build)
		.create_option(user::commands_build)
}

pub fn app_commands_build(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
	commands
		.create_application_command(root_command_build)
}
