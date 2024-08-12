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
    errors,
    session::user_id,
    templates::Channels,
    util::{check_guild_access, get_common},
};

use crate::entities::channel;
use crate::entities::prelude::Channel;
use sea_orm::*;

pub async fn get(
    session: Session,
    Path(guild_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, errors::Error> {
    check_guild_access(guild_id, &state, &session).await?;
    let channels = Channel::find()
        .filter(channel::Column::GuildId.eq(guild_id))
        .order_by_asc(channel::Column::Name)
        .all(state.db.as_ref())
        .await?;

    Ok(Channels {
        common: get_common("Channels", Some(guild_id), &state, &session).await?,
        channels,
        guild_id,
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
) -> Result<Redirect, errors::Error> {
    let _ = user_id(&session).await?;
    channel::ActiveModel {
        id: ActiveValue::NotSet,
        discord_id: ActiveValue::Set(new_channel.channel_id),
        guild_id: ActiveValue::Set(new_channel.guild_id),
        name: ActiveValue::Set(new_channel.name),
        allow_reset: ActiveValue::Set(false),
        reset_message: ActiveValue::NotSet,
        reset_schedule: ActiveValue::NotSet,
    }
    .insert(state.db.as_ref())
    .await?;

    Ok(Redirect::to("/channels"))
}
