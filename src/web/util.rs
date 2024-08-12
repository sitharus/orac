use crate::entities::prelude::{Guild, GuildAccess};
use crate::entities::{guild, guild_access};
use sea_orm::{sea_query::Query, *};
use tower_sessions::Session;

use super::templates::GuildItem;
use super::{appstate::AppState, errors, session::user_id, templates::Common};

pub async fn get_common(
    page_title: &str,
    selected_guild: Option<i32>,
    app_state: &AppState,
    session: &Session,
) -> Result<Common, errors::Error> {
    let user_id = user_id(session).await?;
    let user_guilds = Guild::find()
        .filter(
            Condition::any().add(
                guild::Column::Id.in_subquery(
                    Query::select()
                        .column(guild_access::Column::GuildId)
                        .from(guild_access::Entity)
                        .and_where(guild_access::Column::UserId.eq(user_id))
                        .to_owned(),
                ),
            ),
        )
        .all(app_state.db.as_ref())
        .await?;

    Ok(Common {
        page_title: page_title.to_string(),
        guilds: user_guilds
            .iter()
            .map(|u| GuildItem {
                name: u.name.clone().unwrap_or(u.id.to_string()),
                id: u.id,
                selected: Some(u.id) == selected_guild,
                logo_url: u.logo_url.clone(),
            })
            .collect(),
    })
}

pub async fn check_guild_access(
    guild_id: i32,
    app_state: &AppState,
    session: &Session,
) -> Result<(), errors::Error> {
    let user_id = user_id(session).await?;
    let guild_access = GuildAccess::find()
        .filter(guild_access::Column::GuildId.eq(guild_id))
        .filter(guild_access::Column::UserId.eq(user_id))
        .count(app_state.db.as_ref())
        .await?;

    match guild_access {
        1 => Ok(()),
        _ => Err(errors::Error::Forbidden),
    }
}
