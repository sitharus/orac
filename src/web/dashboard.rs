use axum::response::{IntoResponse, Redirect};
use tower_sessions::Session;

use super::{
    session::user_id,
    templates::{Common, Dashboard},
};

pub async fn dashboard(session: Session) -> Result<impl IntoResponse, Redirect> {
    let _ = user_id(session).await?;
    let page = Dashboard {
        common: Common {
            page_title: "Dashboard",
        },
    };
    Ok(page)
}
