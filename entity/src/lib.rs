pub mod enums;

pub mod confirmed_account;
pub mod guild_config;
pub mod main_account;
pub mod pending_account;
pub mod remind;
pub mod remind_assignee;
pub mod sub_account;
pub mod user_data;
pub mod talk_history;

pub type ConfirmedAccount = confirmed_account::Model;
pub type ConfirmedAccountBehavior = confirmed_account::Entity;

pub type GuildConfig = guild_config::Model;
pub type GuildConfigBehavior = guild_config::Entity;

pub type MainAccount = main_account::Model;
pub type MainAccountBehavior = main_account::Entity;

pub type PendingAccount = pending_account::Model;
pub type PendingAccountBehavior = pending_account::Entity;

pub type SubAccount = sub_account::Model;
pub type SubAccountBehavior = sub_account::Entity;

pub type UserData = user_data::Model;
pub type UserDataBehavior = user_data::Entity;

pub type Remind = remind::Model;
pub type RemindBehavior = remind::Entity;

pub type RemindAssignee = remind_assignee::Model;
pub type RemindAssigneeBehavior = remind_assignee::Entity;

pub type TalkHistory = talk_history::Model;
pub type TalkHistoryBehavior = talk_history::Entity;
