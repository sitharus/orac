use std::env;
use std::sync::Arc;

use lazy_static::lazy_static;
use regex::Regex;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Configuration, StandardFramework};
use serenity::json::json;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use reqwest;
use serde;

use sea_orm::Database;
use sea_orm_migration::prelude::*;

mod entities;
mod migrator;
mod web;

#[group]
#[commands(ping, reset_channel, catfact)]
struct General;

struct Handler {}

#[derive(serde::Deserialize)]
struct CatFact {
    fact: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _msg: Message) {}
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let search: String = "--migrate-only".into();
    let migrate_only = args.contains(&search);

    let db = Database::connect("sqlite:orac.db?mode=rwc").await.unwrap();

    migrator::Migrator::up(&db, None).await.unwrap();

    if migrate_only {
        return;
    }

    let dbcell = Arc::new(db);

    let web_handle = tokio::spawn(async move { web::webserver(dbcell).await });
    match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            let framework = StandardFramework::new().group(&GENERAL_GROUP);
            framework.configure(Configuration::new().prefix("~"));

            // Login with a bot token from the environment
            let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
            let handler = Handler {};

            let mut client = Client::builder(token, intents)
                .event_handler(handler)
                .framework(framework)
                .await
                .expect("Error creating client");

            // start listening for events by starting a single shard
            if let Err(why) = client.start().await {
                println!("An error occurred while running the client: {:?}", why);
            }
            tokio::join!(web_handle).0.unwrap();
        }
        Err(_) => {
            tokio::join!(web_handle).0.unwrap();
        }
    }
}

#[command]
async fn catfact(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Fetching cat fact!");
    let client = reqwest::Client::new();
    let res = client
        .get("https://catfact.ninja/fact")
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?;

    let response = res.json::<CatFact>().await?;

    msg.reply(ctx, response.fact).await?;

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn reset_channel(ctx: &Context, msg: &Message) -> CommandResult {
    lazy_static! {
        static ref CHANNEL_MATCH: Regex = Regex::new(r"<#([0-9]+)>").unwrap();
    }

    let channel_id = CHANNEL_MATCH.captures(&msg.content).unwrap();
    match channel_id.get(1) {
        Some(id) => {
            let channel_id_r = id.as_str().parse::<ChannelId>().unwrap();
            let channel_info = ctx.http.get_channel(channel_id_r).await.unwrap();
            let audit_message = format!("Channel reset requested by #<{}>", msg.author.id);
            match channel_info.guild() {
                Some(channel) => {
                    let new_name = format!("new_{}", channel.name);

                    let options = json!({
                        "name": new_name,
                        "type": channel.kind,
                        "topic": channel.topic,
                        "nsfw": channel.nsfw,
                        "parent_id": channel.parent_id.unwrap(),
                        "permission_overwrites": channel.permission_overwrites,
                        "position": channel.position,

                    });

                    println!("{:?}", channel.guild_id);
                    let options_obj = options.as_object().unwrap();
                    let new_channel_result = ctx
                        .http
                        .create_channel(channel.guild_id, &options_obj, Some(&audit_message))
                        .await;

                    match new_channel_result {
                        Ok(new_channel) => {
                            let rename = json!({
                                "name": channel.name
                            });
                            let rename_obj = rename.as_object().unwrap();
                            ctx.http
                                .delete_channel(channel_id_r, Some(&audit_message))
                                .await?;
                            ctx.http
                                .edit_channel(new_channel.id, &rename_obj, Some(&audit_message))
                                .await?;
                        }
                        Err(e) => {
                            msg.reply(
                                ctx,
                                format!("Resetting #<{}> failed! {:?}", channel_id_r, e),
                            )
                            .await?;
                        }
                    }
                }
                None => {
                    msg.reply(
                        ctx,
                        format!("#<{}> is not a channel I can reset", channel_id_r),
                    )
                    .await?;
                }
            }
        }
        None => {
            msg.reply(
                ctx,
                "I could not find a channel in your message. Usage ~reset_channel #channel",
            )
            .await?;
        }
    }

    Ok(())
}
