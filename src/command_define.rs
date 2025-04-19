use log::error;
use serenity::{
	all::{
		CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateCommandOption,
	},
	async_trait,
};

pub trait BuildCommandOption {
	fn build_command_option(&self) -> CreateCommandOption;
}

impl BuildCommandOption for CommonCommandType {
	fn build_command_option(&self) -> CreateCommandOption {
		match self {
			CommonCommandType::Command(command) => command.command_build(),
			CommonCommandType::SubCommand(sub_command) => sub_command.command_build(),
		}
	}
}

pub trait BaseCommand {
	fn new() -> Self
	where
		Self: Sized;

	fn get_name(&self) -> String;
	fn get_description(&self) -> String;

	fn to_box(self) -> Box<Self>
	where
		Self: Sized, {
		Box::new(self)
	}
}

/*
Paramsは値名→説明→型定義→必須で構成されています
*/
const PARAMS: [(&str, &str, CommandOptionType, bool); 0] = [];

#[async_trait]
pub trait Command: BaseCommand {
	fn args_param(&self) -> &'static [(&'static str, &'static str, CommandOptionType, bool)] {
		&PARAMS
	}

	fn build_args(&self, option: CreateCommandOption) -> CreateCommandOption {
		let mut option = option;

		for (name, desc, option_type, req) in self.args_param() {
			option = option.add_sub_option(CreateCommandOption::new(*option_type, *name, *desc).required(*req));
		}

		option
	}

	async fn execute(
		&self,
		ctx: Context,
		command: CommandInteraction,
		args: Vec<CommandDataOption>,
	) -> serenity::Result<()>;

	fn command_build(&self) -> CreateCommandOption {
		let res = CreateCommandOption::new(CommandOptionType::SubCommand, self.get_name(), self.get_description());

		self.build_args(res)
	}
}

#[async_trait]
pub trait SubCommand: BaseCommand {
	fn get_sub_commands(&self) -> &Vec<CommonCommandType>;
	fn make_sub_commands_option(&self) -> Vec<CreateCommandOption> {
		let mut res = vec![];

		for cmd in self.get_sub_commands() {
			res.push(cmd.build_command_option());
		}

		res
	}

	async fn commands_route(
		&self,
		ctx: Context,
		command: CommandInteraction,
		sub_command: CommandDataOption,
	) -> serenity::Result<()> {
		let mut sub_command_group_option: Option<Vec<CommandDataOption>> = None;
		let mut sub_command_option: Option<Vec<CommandDataOption>> = None;
		if let CommandDataOptionValue::SubCommandGroup(cdos) = sub_command.value {
			sub_command_group_option = Some(cdos);
		} else if let CommandDataOptionValue::SubCommand(cdos) = sub_command.value {
			sub_command_option = Some(cdos);
		}

		if sub_command_group_option.is_none() && sub_command_option.is_none() {
			error!("Unknown Sub command.");
			return Ok(()); // とりあえずOkにしておく
		}

		let mut executed_command = false;
		if let Some(sub_command_group_option) = sub_command_group_option {
			let sub_sub_command = sub_command_group_option[0].to_owned();
			let command_name = sub_sub_command.name.to_string();

			for sub_cmd in self.get_sub_commands() {
				if let CommonCommandType::SubCommand(cmd) = sub_cmd {
					if command_name == cmd.get_name() {
						executed_command = true;
						cmd.commands_route(ctx, command, sub_sub_command).await?;
						break;
					}
				}
			}
		} else {
			let sub_command_option = sub_command_option.unwrap();
			let command_name = sub_command.name.to_string();

			for cmd in self.get_sub_commands() {
				if let CommonCommandType::Command(cmd) = cmd {
					if command_name == cmd.get_name() {
						executed_command = true;
						cmd.execute(ctx, command, sub_command_option).await?;
						break;
					}
				}
			}
		}

		if !executed_command {
			error!("No Exist Command!");
		}

		Ok(())
	}

	fn command_build(&self) -> CreateCommandOption {
		let mut co = CreateCommandOption::new(
			CommandOptionType::SubCommandGroup,
			self.get_name(),
			self.get_description(),
		);

		for sub_command in self.make_sub_commands_option() {
			co = co.add_sub_option(sub_command);
		}

		co
	}
}

pub enum CommonCommandType {
	Command(Box<dyn Command + Sync + Send>),
	SubCommand(Box<dyn SubCommand + Sync + Send>),
}

impl Into<CommonCommandType> for Box<dyn SubCommand + Sync + Send> {
	fn into(self) -> CommonCommandType {
		CommonCommandType::SubCommand(self)
	}
}

impl Into<CommonCommandType> for Box<dyn Command + Sync + Send> {
	fn into(self) -> CommonCommandType {
		CommonCommandType::Command(self)
	}
}
