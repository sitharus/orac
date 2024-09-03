use poise::serenity_prelude as serenity;
use poise::FrameworkContext;
use sea_orm::sea_query::OnConflict;

use crate::entities::guild;
use sea_orm::*;

use crate::{Data, Error};

pub async fn handle_event(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::GuildCreate { guild, is_new: _ } => {
            let guild_id: String = guild.id.to_string();

            let record = guild::ActiveModel {
                id: ActiveValue::NotSet,
                discord_id: ActiveValue::Set(guild_id),
                name: ActiveValue::Set(Some(guild.name.clone())),
                logo_url: ActiveValue::Set(guild.icon_url()),
            };

            guild::Entity::insert(record)
                .on_conflict(
                    OnConflict::column(guild::Column::DiscordId)
                        .update_columns([guild::Column::Name, guild::Column::LogoUrl])
                        .to_owned(),
                )
                .exec(data.db.as_ref())
                .await?;
            Ok(())
        }
        _ => {
            println!(
                "Got an event in event handler: {:?}",
                event.snake_case_name()
            );
            Ok(())
        }
    }
}
