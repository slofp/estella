use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("ok")
		.description("estella.test.ok")
		.kind(ApplicationCommandOptionType::SubCommand)
		.create_sub_option(|param_option| {
			param_option
				.name("val")
				.description("estella.test.ok value")
				.kind(ApplicationCommandOptionType::String)
		})
}