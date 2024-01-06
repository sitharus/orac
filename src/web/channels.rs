use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

use super::{
    appstate::AppState,
    session::user_id,
    templates::{Channels, Common},
};

use crate::entities::prelude::{Channel, Guild};
use crate::entities::{channel, guild};
use sea_orm::*;

pub async fn get(
    session: Session,

    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Redirect> {
    let _ = user_id(session).await?;
    let channels = Channel::find()
        .order_by_asc(channel::Column::Name)
        .all(state.db.as_ref())
        .await
        .expect("Could not load channels");
    let guilds = Guild::find()
        .order_by_asc(guild::Column::Name)
        .all(state.db.as_ref())
        .await
        .expect("Could not load guilds");

    let guild_map = HashMap::from_iter(guilds.clone().into_iter().map(|g| (g.id, g)));

    Ok(Channels {
        common: Common {
            page_title: "Channels".into(),
        },
        channels,
        guilds,
        guild_map,
    })
}

#[derive(Deserialize)]
pub struct NewChannelForm {
    guild_id: i32,
    channel_id: i64,
    name: Option<String>,
}

pub async fn post(
    session: Session,
    State(state): State<Arc<AppState>>,
    Form(new_channel): Form<NewChannelForm>,
) -> Result<Redirect, Redirect> {
    let _ = user_id(session).await?;
    channel::ActiveModel {
        id: ActiveValue::NotSet,
        discord_id: ActiveValue::Set(new_channel.channel_id),
        guild_id: ActiveValue::Set(new_channel.guild_id),
        name: ActiveValue::Set(new_channel.name),
    }
    .insert(state.db.as_ref())
    .await
    .expect("Could not save channel");

    Ok(Redirect::to("/channels"))
}

pub async fn get_channel(
    Path(channel_id): Path<i32>,
    session: Session,
) -> Result<Redirect, Redirect> {
    let _ = user_id(session).await?;
    Ok(Redirect::to("/channels"))
}
