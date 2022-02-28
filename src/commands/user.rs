use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

pub mod reserve;

pub fn commands_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("user")
		.description("Estella User Commands")
		.kind(ApplicationCommandOptionType::SubCommandGroup)
		.create_sub_option(reserve::command_build)
}
