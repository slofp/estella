use log::error;
use serenity::builder::CreateApplicationCommandOption;
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType};
use serenity::model::interactions::{InteractionApplicationCommandCallbackDataFlags, InteractionResponseType};

const PARAM_ADDRESS: &str = "address";

/*
Paramsは値名→説明→型定義→必須で構成されています
*/
const PARAMS: [(&str, &str, ApplicationCommandOptionType, bool); 1] = [
	(PARAM_ADDRESS, "メールアドレス(アットマーク以降は不要)", ApplicationCommandOptionType::String, true)
];

pub async fn execute(ctx: Context, command: &ApplicationCommandInteraction, command_args: &ApplicationCommandInteractionDataOption) {
	let mut address_o: Option<String> = None;

	for option in &command_args.options {
		//info!("option data: {} [{:?}]", b.name, b.value);
		match (&option.name).as_str() {
			PARAM_ADDRESS => {
				let option_value = option.value.as_ref().unwrap();
				if option_value.is_string() {
					address_o = Some(option_value.as_str().unwrap_or_else(|| "").to_string());
				}
			},
			_ => {}
		}
	}

	if address_o.is_none() {
		error!("Address is undefined.");
	}

	let address = address_o.unwrap();

}

pub fn command_build(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
	option
		.name("create")
		.description("メールアドレスを作成します")
		.kind(ApplicationCommandOptionType::SubCommand);

	for (name, desc, option_type, req) in &PARAMS {
		option
			.create_sub_option(|param_option| {
				param_option
					.name(name)
					.description(desc)
					.kind(*option_type)
					.required(*req)
			});
	}

	return option;
}
