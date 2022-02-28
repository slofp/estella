use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};

pub mod test;

fn root_command_build(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("estella")
		.description("Estella Command Root")
		.create_option(test::commands_build)
}

pub fn app_commands_build(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
	commands
		.create_application_command(root_command_build)
}
