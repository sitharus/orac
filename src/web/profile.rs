use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::entities::prelude::User;
use sea_orm::*;

use super::{
    appstate::AppState,
    session::user_id,
    templates::{Common, Profile},
};

#[derive(Deserialize)]
pub struct ProfileForm {
    name: Option<String>,
    email: Option<String>,
    discord_id: Option<i64>,
    current_password: Option<String>,
    new_password: Option<String>,
    repeat_password: Option<String>,
}

pub async fn profile(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Redirect> {
    let id = user_id(session).await?;
    let user = User::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| Redirect::to("/"))?
        .expect("Managed to get a user id that doesn't exist?");

    let page = Profile {
        common: Common {
            page_title: "Profile",
        },
        name: user.name,
        email: user.email,
        discord_id: user.discord_id,
    };
    Ok(page)
}

pub async fn submit(
    session: Session,
    State(state): State<Arc<AppState>>,
    Form(update): Form<ProfileForm>,
) -> Result<impl IntoResponse, Redirect> {
    let id = user_id(session).await?;
    let user = User::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| Redirect::to("/"))?
        .expect("Managed to get a user id that doesn't exist?");

    match update.current_password {
        Some(_) => Ok(Redirect::to("/profile")),
        None => {
            let mut user = user.into_active_model();
            user.email = Set(update.email.expect("Email is required"));
            user.name = Set(update.name.expect("Name is required"));
            user.discord_id = Set(update.discord_id);
            user.update(state.db.as_ref())
                .await
                .expect("Db update failed");

            Ok(Redirect::to("/profile"))
        }
    }
}
