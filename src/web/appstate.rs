use std::sync::Arc;

use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;

use crate::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: Config,
    pub discord: Arc<serenity::http::Http>,
}
