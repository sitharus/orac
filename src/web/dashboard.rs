use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use tower_sessions::Session;

use super::{appstate::AppState, errors, session::user_id, templates::Dashboard, util::get_common};

pub async fn dashboard(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, errors::Error> {
    let _ = user_id(&session).await?;
    let page = Dashboard {
        common: get_common("Dashboard", None, &state, &session).await?,
    };
    Ok(page)
}
