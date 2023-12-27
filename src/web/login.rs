use std::sync::Arc;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use sea_orm::*;
use serde::Deserialize;

use super::appstate::AppState;
use super::session::USER_ID_KEY;
use super::templates::IndexTemplate;
use crate::entities::{prelude::*, *};
use tower_sessions::Session;

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login(
    session: Session,
    State(state): State<Arc<AppState>>,
    Form(login_form): Form<LoginForm>,
) -> impl IntoResponse {
    let connection: &DatabaseConnection = state.db.as_ref();
    let username = login_form.username;
    let user = User::find()
        .filter(user::Column::Email.eq(&username))
        .one(connection)
        .await;

    match user {
        Ok(Some(user)) => match validate_password(&user, &login_form.password, connection).await {
            Ok(true) => {
                session.insert(USER_ID_KEY, user.id).await.unwrap();
                Err(Redirect::to("/dashboard"))
            }
            _ => Ok(IndexTemplate {
                username: Some(username.into()),
                message: Some("Invalid password".into()),
            }),
        },
        e => {
            eprintln!("{:?}", e);
            Err(Redirect::to("/"))
        }
    }
}

async fn validate_password(
    user: &user::Model,
    password: &String,
    db: &DatabaseConnection,
) -> anyhow::Result<bool> {
    let pwbytes = password.as_bytes();

    if let Ok(success) = bcrypt::verify(&pwbytes, &user.password) {
        Ok(success)
    } else if &user.password == password {
        let hashed = bcrypt::hash(&user.password.clone(), bcrypt::DEFAULT_COST)?;
        let mut user: user::ActiveModel = user.to_owned().into();
        user.password = Set(hashed);
        user.update(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
