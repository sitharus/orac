use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use serenity::all::{ChannelType, GuildChannel};
use tower_sessions::Session;

use super::{
    appstate::AppState,
    errors,
    session::user_id,
    templates::{AddChannel, Channels},
    util::{check_guild_access, get_common, get_guild_for_user},
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

pub async fn add(
    session: Session,
    Path(guild_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, errors::Error> {
    let guild = get_guild_for_user(guild_id, &state, &session).await?;
    let existing_channels = Channel::find()
        .filter(channel::Column::GuildId.eq(guild_id))
        .all(state.db.as_ref())
        .await?;
    let existing_channel_ids: Vec<String> = existing_channels
        .into_iter()
        .map(|c| c.discord_id)
        .collect();
    let mut channels: Vec<GuildChannel> = state
        .discord
        .get_channels(guild.discord_id.parse()?)
        .await?
        .into_iter()
        .filter(|c| {
            c.kind == ChannelType::Text && !existing_channel_ids.contains(&c.id.to_string())
        })
        .collect();

    channels.sort_by(|a, b| a.position.cmp(&b.position));

    let page = AddChannel {
        common: get_common("Add Channel", Some(guild_id), &state, &session).await?,
        channels,
        guild_id,
    };

    Ok(page)
}
#[derive(Deserialize)]
pub struct AddChannelForm {
    channel: String,
    auto_reset: bool,
    auto_reset_cron: Option<String>,
    auto_reset_message: Option<String>,
}

pub async fn add_post(
    session: Session,
    Path(guild_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Form(add): Form<AddChannelForm>,
) -> Result<impl IntoResponse, errors::Error> {
    let guild = get_guild_for_user(guild_id, &state, &session).await?;
    let channel_detail = state.discord.get_channel(add.channel.parse()?).await?;

    match channel_detail {
        serenity::model::channel::Channel::Guild(c)
            if c.guild_id.to_string() == guild.discord_id =>
        {
            let new_channel = channel::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set(Some(c.name)),
                guild_id: ActiveValue::Set(guild_id),
                discord_id: ActiveValue::Set(c.id.to_string()),
                allow_reset: ActiveValue::Set(add.auto_reset),
                reset_message: ActiveValue::Set(add.auto_reset_message),
                reset_schedule: ActiveValue::Set(add.auto_reset_cron),
            };

            channel::Entity::insert(new_channel)
                .exec(state.db.as_ref())
                .await?;
            Ok(Redirect::to(
                format!("/guild/{}/channels", guild_id).as_str(),
            ))
        }
        _ => Err(errors::Error::Forbidden),
    }
}
