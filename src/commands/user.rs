use find::FindCommand;
use reserve::ReserveCommand;
use sub_application::SubApplicationCommand;

use crate::command_define::{BaseCommand, CommonCommandType, SubCommand};

mod find;
mod reserve;
mod sub_application;

pub struct UserCommands {
	sub_commands: Vec<CommonCommandType>,
}

impl BaseCommand for UserCommands {
	fn new() -> Self {
		Self {
			sub_commands: vec![
				convert_command!(ReserveCommand),
				convert_command!(SubApplicationCommand),
				convert_command!(FindCommand),
			],
		}
	}

	fn get_name(&self) -> String {
		"user".into()
	}

	fn get_description(&self) -> String {
		"Estella User Commands".into()
	}
}

impl SubCommand for UserCommands {
	fn get_sub_commands(&self) -> &Vec<CommonCommandType> {
		&self.sub_commands
	}
}
