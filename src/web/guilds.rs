use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tower_sessions::Session;

use super::{
    appstate::AppState, errors::Error, session::user_id, templates::GuildPage, util::get_common,
};

use crate::entities::channel;
use crate::entities::prelude::{Channel, Guild};
use sea_orm::*;

pub async fn get(
    session: Session,
    Path(guild_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let _ = user_id(&session).await?;
    let guild = Guild::find_by_id(guild_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(anyhow::anyhow!("Could not load guild!"))?;
    let common = get_common(
        guild
            .name
            .clone()
            .unwrap_or(guild.id.to_string().into())
            .as_str(),
        None,
        &state,
        &session,
    )
    .await?;

    let managed_channels = Channel::find()
        .filter(channel::Column::GuildId.eq(guild.id))
        .order_by(channel::Column::Name, Order::Asc)
        .all(state.db.as_ref())
        .await?;

    Ok(GuildPage {
        common,
        guild,
        managed_channels,
    })
}
