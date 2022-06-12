use chrono::{DateTime, Local, Utc};

pub fn username(name: String, discriminator: u16) -> String {
	format!("{}#{}", name, discriminator)
}

pub fn utc_to_local_format(time: &DateTime<Utc>) -> String {
	time.with_timezone(&Local).format("%Y/%m/%d %H:%M:%S").to_string()
}
