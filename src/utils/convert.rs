use std::error::Error;
use chrono::{DateTime, Local, Utc};
use serenity::all::User;

pub fn format_discord_username(user: &User) -> String {
	format!("{}{}", user.name, match user.discriminator {
		None => String::new(),
		Some(n) => format!("#{:04}", n)
	})
}

pub fn utc_to_local_format(time: &DateTime<Utc>) -> String {
	time.with_timezone(&Local).format("%Y/%m/%d %H:%M:%S").to_string()
}

pub fn flatten_result_option<T, E: Error>(value: Result<Option<T>, E>) -> Result<T, E> {
	match value {
		Ok(v) => v?,
		Err(e) => Err(e)
	}
}
