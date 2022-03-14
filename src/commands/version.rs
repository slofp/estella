use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("version")
		.description("Estellaのバージョンを表示します")
		.kind(ApplicationCommandOptionType::SubCommand)
}
