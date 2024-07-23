use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: Config,
}
