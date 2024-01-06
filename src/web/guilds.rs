use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

use super::{
    appstate::AppState,
    session::user_id,
    templates::{Common, Guilds},
};

use crate::entities::guild;
use crate::entities::prelude::Guild;
use sea_orm::*;

pub async fn get(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Redirect> {
    let _ = user_id(session).await?;

    let guilds = Guild::find()
        .all(state.db.as_ref())
        .await
        .expect("Could not load guilds");

    Ok(Guilds {
        common: Common {
            page_title: "Guilds",
        },
        guilds,
    })
}

#[derive(Deserialize)]
pub struct GuildForm {
    guild_id: String,
    name: String,
}

pub async fn post(
    session: Session,
    State(state): State<Arc<AppState>>,
    Form(new_guild): Form<GuildForm>,
) -> Result<Redirect, Redirect> {
    let _ = user_id(session).await?;
    guild::ActiveModel {
        id: ActiveValue::NotSet,
        discord_id: ActiveValue::Set(new_guild.guild_id.parse::<i64>().expect("Not a number")),
        name: ActiveValue::Set(Some(new_guild.name)),
    }
    .insert(state.db.as_ref())
    .await
    .expect("Could not save guild");

    Ok(Redirect::to("/guilds"))
}
