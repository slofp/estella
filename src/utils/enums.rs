#[derive(sqlx::Type)]
#[repr(u8)]
pub enum AccountType {
	Main = 1,
	Sub = 2,
}

impl AccountType {
	pub fn to_string(&self) -> String {
		match self {
			AccountType::Main => "メイン",
			AccountType::Sub => "サブ"
		}.to_string()
	}
}

pub enum ConfResponseType {
	Ok = 1,
	EqualErr = 2,
	ExistErr = 3,
	OtherErr = 4,
	Success = 5,
}
