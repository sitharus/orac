use sea_orm_migration::prelude::*;

mod m20231217_000001_create_user_and_sessions_table;
mod m20231226_112401_session_store;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231217_000001_create_user_and_sessions_table::Migration),
            Box::new(m20231226_112401_session_store::Migration),
        ]
    }
}
