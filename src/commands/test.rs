use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

pub mod ok;

pub fn commands_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("test")
		.description("estella.test")
		.kind(ApplicationCommandOptionType::SubCommandGroup)
		.create_sub_option(ok::command_build)
}
