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
mod login;
mod session;
mod templates;

use appstate::AppState;

use self::session::OrmStore;

pub async fn webserver(database: Arc<DatabaseConnection>) {
    let session_store = OrmStore::new(database.clone());

    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(10))),
        );

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("assets"))
        .route("/login", post(login::login))
        .route("/dashboard", get(dashboard::dashboard))
        .layer(session_service)
        .with_state(Arc::new(AppState { db: database }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    templates::IndexTemplate {
        message: None,
        username: None,
    }
}
