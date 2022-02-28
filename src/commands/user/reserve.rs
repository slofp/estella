use serenity::builder::CreateApplicationCommandOption;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

/*
Paramsは値名→説明→型定義で構成されています
*/
const PARAMS: [(&str, &str, ApplicationCommandOptionType); 3] = [
	("user_id", "ユーザーID", ApplicationCommandOptionType::String),
	("name", "登録名", ApplicationCommandOptionType::String),
	("reason", "登録理由", ApplicationCommandOptionType::String),
];

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("reserve")
		.description("ユーザー登録を予約します")
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
