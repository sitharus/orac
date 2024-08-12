use sea_orm_migration::prelude::*;
mod entities;

mod m20231217_000001_create_user_and_sessions_table;
mod m20231226_112401_session_store;
mod m20231227_113307_channel_store;
mod m20231227_113316_user_store;
mod m20240106_032511_channel_settings;
mod m20240724_073046_unique_guild;
mod m20240724_090901_guild_access;
mod m20240729_100648_channel_options;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231217_000001_create_user_and_sessions_table::Migration),
            Box::new(m20231226_112401_session_store::Migration),
            Box::new(m20231227_113307_channel_store::Migration),
            Box::new(m20231227_113316_user_store::Migration),
            Box::new(m20240106_032511_channel_settings::Migration),
            Box::new(m20240724_073046_unique_guild::Migration),
            Box::new(m20240724_090901_guild_access::Migration),
            Box::new(m20240729_100648_channel_options::Migration),
        ]
    }
}
