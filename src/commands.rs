use crate::{Context, Error};
use ::serenity::builder::Builder;
use poise::serenity_prelude as serenity;

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Orac is maintained by Thea, so blame her.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct CatFact {
    fact: String,
}

/// Fetches a cat fact
#[poise::command(slash_command)]
pub async fn catfact(ctx: Context<'_>) -> Result<(), Error> {
    println!("Fetching cat fact!");
    let client = reqwest::Client::new();
    let res = client
        .get("https://catfact.ninja/fact")
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?;

    let response = res.json::<CatFact>().await?;

    ctx.reply(response.fact).await?;

    Ok(())
}

/// Ping-pong!
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("pong").await?;
    Ok(())
}

/// Resets a channel by deleting and re-creating it
#[poise::command(slash_command, owners_only, hide_in_help)]
pub async fn reset_channel(
    ctx: Context<'_>,

    #[description = "The channel to reset"]
    #[channel_types("Text")]
    channel: serenity::Channel,
) -> Result<(), Error> {
    match channel.guild() {
        Some(guild_channel) => {
            let new_name = format!("new_{}", guild_channel.name);
            let http = ctx.http();
            let builder = serenity::builder::CreateChannel::new(new_name)
                .kind(guild_channel.kind)
                .topic(guild_channel.topic.clone().unwrap_or_default())
                .nsfw(guild_channel.nsfw)
                .category(guild_channel.parent_id.unwrap_or_default())
                .permissions(guild_channel.permission_overwrites.clone())
                .position(guild_channel.position);
            let mut result = builder.execute(http, guild_channel.guild_id).await?;
            guild_channel.delete(ctx.http()).await?;

            result
                .edit(
                    http,
                    serenity::builder::EditChannel::new().name(guild_channel.name),
                )
                .await?;

            ctx.say(format!("<#{}> has been reset!", result.id)).await?;

            Ok(())
        }
        _ => Ok(()),
    }
}
