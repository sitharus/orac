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
    errors,
    session::{user_id, user_id_or_redirect},
    templates::Profile,
    util::get_common,
};

#[derive(Deserialize)]
pub struct ProfileForm {
    name: Option<String>,
    email: Option<String>,
}

pub async fn profile(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, errors::Error> {
    let id = user_id(&session).await?;
    let user = User::find_by_id(id)
        .one(state.db.as_ref())
        .await?
        .ok_or(anyhow::anyhow!("Your user doesn't exist? Clever!"))?;

    let common = get_common("Profile", None, &state, &session).await?;
    let page = Profile {
        common,
        name: user.name,
        email: user.email,
    };
    Ok(page)
}

pub async fn submit(
    session: Session,
    State(state): State<Arc<AppState>>,
    Form(update): Form<ProfileForm>,
) -> Result<impl IntoResponse, Redirect> {
    let id = user_id_or_redirect(session).await?;
    let user = User::find_by_id(id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| Redirect::to("/"))?
        .expect("Managed to get a user id that doesn't exist?");

    let mut user = user.into_active_model();
    user.email = Set(update.email.expect("Email is required"));
    user.name = Set(update.name.expect("Name is required"));
    user.update(state.db.as_ref())
        .await
        .expect("Db update failed");

    Ok(Redirect::to("/profile"))
}
