use std::sync::Arc;
use std::{env, time::Duration};

use poise::serenity_prelude as serenity;

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;

mod commands;
mod entities;
mod migrator;
mod web;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub db: Arc<DatabaseConnection>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    discord_token: String,
    client_secret: String,
    client_id: String,
    redirect_url: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
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

    let config_str = std::fs::read_to_string("bot_config.toml")
        .expect("Could not read config from bot_config.toml");
    let config = toml::from_str::<Config>(config_str.as_ref()).expect("Could not parse config!");

    let dbcell = Arc::new(db);
    // will dbcell ever get to 0 refs? I think so because this will be moved
    // to the closure, then released once that exits. But it doesn't really
    // matter because the db connection pretty much has 'static lifetime.
    let poise_cell = dbcell.clone();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::catfact(),
            commands::ping(),
            commands::reset_channel(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db: poise_cell })
            })
        })
        .options(options)
        .build();

    let web_config = config.clone();
    let web_handle = tokio::spawn(async move { web::webserver(dbcell, web_config).await });

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    tokio::join!(web_handle).0.unwrap();
}
