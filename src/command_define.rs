use log::error;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateCommandOption};
use serenity::async_trait;

pub trait BaseCommand {
	fn new() -> Self;

	fn get_name(&self) -> impl Into<String>;
	fn get_description(&self) -> impl Into<String>;

	fn build_command_option(&self: impl Into<CommonCommandType>) -> CreateCommandOption {
		match self.into() {
			CommonCommandType::Command(c) => {
				c.command_build()
			}
			CommonCommandType::SubCommand(sc) => {
				sc.command_build()
			}
		}
	}
}

#[async_trait]
pub trait Command: BaseCommand {
	fn build_args(&self, option: CreateCommandOption) -> CreateCommandOption {
		option
	}

	async fn execute(&self, ctx: Context, command: CommandInteraction, args: Vec<CommandDataOption>) -> serenity::Result<()> ;

	fn command_build(&self) -> CreateCommandOption {
		let res = CreateCommandOption::new(
			CommandOptionType::SubCommand,
			self.get_name(),
			self.get_description()
		);

		self.build_args(res)
	}
}

#[async_trait]
pub trait SubCommand: BaseCommand {
	fn get_sub_commands(&self) -> Vec<CommonCommandType>;
	fn get_sub_commands_option(&self) -> Vec<CreateCommandOption>;

	async fn commands_route(&self, ctx: Context, command: CommandInteraction, sub_command: CommandDataOption) -> serenity::Result<()>  {
		let mut sub_command_group_option: Option<Vec<CommandDataOption>> = None;
		let mut sub_command_option: Option<Vec<CommandDataOption>> = None;
		if let CommandDataOptionValue::SubCommandGroup(cdos) = sub_command.value {
			sub_command_group_option = Some(cdos);
		}
		else if let CommandDataOptionValue::SubCommand(cdos) = sub_command.value {
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
				if matches!(sub_cmd, CommonCommandType::Command(_)) {
					continue;
				}
				let CommonCommandType::SubCommand(cmd) = sub_cmd;

				if command_name == cmd.get_name() {
					executed_command = true;
					cmd.commands_route(ctx, command, sub_sub_command).await?;
					break;
				}
			}
		}
		else {
			let sub_command_option = sub_command_option.unwrap();
			let command_name = sub_command.name.to_string();

			for cmd in self.get_sub_commands() {
				if matches!(cmd, CommonCommandType::SubCommand(_)) {
					continue;
				}
				let CommonCommandType::Command(cmd) = cmd;

				if command_name == cmd.get_name() {
					executed_command = true;
					cmd.execute(ctx, command, sub_command_option).await?;
					break;
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
			self.get_description()
		);

		for sub_command in self.get_sub_commands_option() {
			co = co.add_sub_option(sub_command);
		}

		co
	}
}

pub enum CommonCommandType {
	Command(dyn Command),
	SubCommand(dyn SubCommand)
}

impl From<dyn Command> for CommonCommandType {
	fn from(value: impl Command) -> Self {
		CommonCommandType::Command(value)
	}
}

impl From<dyn SubCommand> for CommonCommandType {
	fn from(value: impl SubCommand) -> Self {
		CommonCommandType::SubCommand(value)
	}
}
