use chrono::{DateTime, Local, Utc};
use serenity::all::User;
use std::{error::Error, fmt::Display};

pub fn format_discord_username(user: &User) -> String {
	format!(
		"{}{}",
		user.name,
		match user.discriminator {
			None => String::new(),
			Some(n) => format!("#{:04}", n),
		}
	)
}

pub fn utc_to_local_format(time: &DateTime<Utc>) -> String {
	time.with_timezone(&Local).format("%Y/%m/%d %H:%M:%S").to_string()
}

pub fn flatten_result_option<'a, T, E: Error + Sync + Send + 'a>(
	value: Result<Option<T>, E>,
) -> Result<T, Box<dyn Error + Sync + Send + 'a>> {
	match value {
		Ok(v) => v.ok_or_else(|| Box::new(OptionalNoneError::new()) as Box<dyn Error + Sync + Send>),
		Err(e) => Err(Box::new(e) as Box<dyn Error + Sync + Send>),
	}
}

#[derive(Debug)]
pub struct OptionalNoneError;

impl OptionalNoneError {
	fn new() -> Self {
		OptionalNoneError
	}
}

impl Error for OptionalNoneError {}

impl Display for OptionalNoneError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Option<T> is not found.")
	}
}
