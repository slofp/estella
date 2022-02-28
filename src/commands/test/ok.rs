use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

const PARAMS: [(&str, &str, ApplicationCommandOptionType); 1] = [
	("val", "estella.test.ok value", ApplicationCommandOptionType::String)
];

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("ok")
		.description("estella.test.ok")
		.kind(ApplicationCommandOptionType::SubCommand);

	for (name, desc, option_type) in &PARAMS {
		option
			.create_sub_option(|param_option| {
				param_option
					.name(name)
					.description(desc)
					.kind(*option_type)
			});
	}

	return option;
}
