use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum GuildConfig {
	Table,

	// Column
	Uid,
	WhiteList,
	LeaveBan,
	LogChannelId,
	AuthRoleId,
	BotRoleId,

	// 2025-06-05 added
	SendAiChatChannelId,
}

#[derive(DeriveIden)]
pub enum MainAccount {
	Table,

	// Column
	Uid,
	Name,
	GuildId,
	Version,
	JoinDate,
	IsServerCreator,
	IsLeaved,
}

#[derive(DeriveIden)]
pub enum SubAccount {
	Table,

	// Column
	Uid,
	Name,
	GuildId,
	JoinDate,
	MainUid,
	FirstCert,
	SecondCert,
}

#[derive(DeriveIden)]
pub enum ConfirmedAccount {
	Table,

	// Column
	Uid,
	Name,
	GuildId,
	AccountType,
	MainUid,
	FirstCert,
	SecondCert,
}

#[derive(DeriveIden)]
pub enum PendingAccount {
	Table,

	// Column
	Uid,
	Name,
	GuildId,
	AccountType,
	MessageId,
	EndVoting,
	MainUid,
	FirstCert,
}

#[derive(DeriveIden)]
pub enum UserData {
	Table,

	// Column
	Uid,
	Glacialeur,

	// 2025-06-02 added
	CallName,
	Gender,
	LikabilityLevel,
}

#[derive(DeriveIden)]
pub enum Remind {
	Table,

	// Column
	Id,
	TaskName,
	AuthorId,
	AssigneesId,
	RemindDate,
}

#[derive(DeriveIden)]
pub enum RemindAssignee {
	Table,

	// Column
	Id,
	UserId,
}
