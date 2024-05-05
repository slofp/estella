pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240503_165913_create_table_guild_config;
mod m20240503_165926_create_table_main_account;
mod m20240503_165935_create_table_sub_account;
mod m20240503_165947_create_table_confirmed_account;
mod m20240503_165956_create_table_pending_account;
mod m20240503_170005_create_table_user_data;
mod tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240503_165913_create_table_guild_config::Migration),
            Box::new(m20240503_165926_create_table_main_account::Migration),
            Box::new(m20240503_165935_create_table_sub_account::Migration),
            Box::new(m20240503_165947_create_table_confirmed_account::Migration),
            Box::new(m20240503_165956_create_table_pending_account::Migration),
            Box::new(m20240503_170005_create_table_user_data::Migration),
        ]
    }
}
