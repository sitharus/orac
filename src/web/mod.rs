use axum::{
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use time::Duration;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, Session, SessionManagerLayer};

mod appstate;
mod channels;
mod dashboard;
mod errors;
mod filters;
mod guilds;
mod oauth;
mod profile;
mod session;
mod templates;
mod util;

use appstate::AppState;

use crate::Config;

use self::session::{user_id, OrmStore};

pub async fn webserver(database: Arc<DatabaseConnection>, config: Config) {
    let session_store = OrmStore::new(database.clone());

    let discord_client = serenity::http::Http::new(&config.discord_token);

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("assets"))
        .route("/oauth/start", get(oauth::start_oauth))
        .route("/oauth/token", get(oauth::oauth_login))
        .route("/oauth/select_guild", get(oauth::select_guild))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/profile", get(profile::profile).post(profile::submit))
        .route("/guild/:guild_id", get(guilds::get))
        .route(
            "/guild/:guild_id/channels",
            get(channels::get).post(channels::post),
        )
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_same_site(tower_sessions::cookie::SameSite::Lax)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(1200))),
        )
        .with_state(Arc::new(AppState {
            db: database,
            config,
            discord: Arc::new(discord_client),
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index(session: Session) -> Result<impl IntoResponse, errors::Error> {
    let user_id = user_id(&session).await;
    match user_id {
        Ok(_) => Ok(Redirect::to("/dashboard").into_response()),
        _ => {
            session.insert("TEST", "TEST").await?;
            Ok(templates::IndexTemplate {
                message: None,
                username: None,
            }
            .into_response())
        }
    }
}
