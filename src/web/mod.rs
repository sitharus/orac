use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use time::Duration;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, SessionManagerLayer};

mod appstate;
mod channels;
mod dashboard;
mod errors;
mod filters;
mod guilds;
mod login;
mod oauth;
mod profile;
mod session;
mod templates;

use appstate::AppState;

use crate::Config;

use self::session::OrmStore;

pub async fn webserver(database: Arc<DatabaseConnection>, config: Config) {
    let session_store = OrmStore::new(database.clone());

    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(1200))),
        );

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("assets"))
        .route("/login", post(login::login))
        .route("/oauth/start", get(oauth::start_oauth))
        .route("/oauth/token", get(oauth::oauth_login))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/profile", get(profile::profile).post(profile::submit))
        .route("/guilds", get(guilds::get).post(guilds::post))
        .route("/channels", get(channels::get).post(channels::post))
        .route("/channel/:channel_id", get(channels::get_channel))
        .layer(session_service)
        .with_state(Arc::new(AppState {
            db: database,
            config,
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    templates::IndexTemplate {
        message: None,
        username: None,
    }
}
