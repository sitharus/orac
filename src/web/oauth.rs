use std::collections::HashMap;
use std::sync::Arc;

use crate::entities::guild_access::RoleInfo;
use crate::entities::{prelude::*, *};
use crate::web::session::USER_ID_KEY;
use crate::Config;
use anyhow;
use axum::extract::{Query, State};
use axum::response::Redirect;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use sea_orm::sea_query::OnConflict;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::appstate::AppState;
use super::errors;
use super::session::{discord_user_id, user_id, DISCORD_USER_ID_KEY};

#[derive(Serialize, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    global_name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct PartialGuild {
    id: String,
    name: String,
    owner: bool,
    permissions: i64,
}

pub async fn start_oauth(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Redirect, errors::Error> {
    // Generate a PKCE challenge.
    session
        .remove::<serde_json::Value>(DISCORD_USER_ID_KEY)
        .await?;
    session.remove::<serde_json::Value>(USER_ID_KEY).await?;
    session.remove::<serde_json::Value>("OAUTH_TOKEN").await?;

    // Generate the full authorization URL.
    let (auth_url, _csrf_token) = oauth_client(&state.config)?
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("guilds".to_string()))
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn oauth_login(
    Query(params): Query<HashMap<String, String>>,
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, errors::Error> {
    let token_param = params.get("code").ok_or(anyhow::anyhow!("No OAuth code"))?;
    let token_result = oauth_client(&state.config)?
        .exchange_code(AuthorizationCode::new(token_param.to_string()))
        .request_async(async_http_client)
        .await;
    let unwrapped = token_result?;
    let access_token = unwrapped.access_token();

    let client = reqwest::Client::new();
    let me = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token.secret()))
        .send()
        .await?
        .json::<DiscordUser>()
        .await?;
    let connection: &DatabaseConnection = state.db.as_ref();
    let user = User::find()
        .filter(user::Column::Email.eq(me.email))
        .one(connection)
        .await?;

    let discord_user_id = Some(me.id.clone());

    session.insert("OAUTH_TOKEN", unwrapped).await?;
    match user {
        Some(u) => {
            session.insert(USER_ID_KEY, u.id).await?;
            session.insert(DISCORD_USER_ID_KEY, me.id).await?;
            if u.discord_id != discord_user_id {
                let mut user_model: user::ActiveModel = u.into();
                user_model.discord_id = Set(discord_user_id);
                user_model.update(state.db.as_ref()).await?;
            }

            Ok(Redirect::to("/oauth/select_guild"))
        }
        None => Ok(Redirect::to("/")),
    }
}

pub async fn select_guild(
    session: Session,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, errors::Error> {
    session.load().await?;

    let client = reqwest::Client::new();
    let oauth_token = session
        .get::<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>("OAUTH_TOKEN")
        .await?
        .ok_or(anyhow::anyhow!("No oauth token!"))?;

    let access_token = oauth_token.access_token();
    let user_id = user_id(&session).await?;
    let discord_user_id = discord_user_id(&session).await?;

    let user_guilds = client
        .get("https://discord.com/api/users/@me/guilds")
        .header("Authorization", format!("Bearer {}", access_token.secret()))
        .send()
        .await?
        .json::<Vec<PartialGuild>>()
        .await?;
    let bot_guilds = Guild::find().all(state.db.as_ref()).await?;

    guild_access::Entity::delete_many()
        .filter(guild_access::Column::UserId.eq(user_id))
        .exec(state.db.as_ref())
        .await?;

    for guild in user_guilds {
        let guild_id = guild.id;
        if let Some(guild_record) = bot_guilds.iter().find(|&g| g.discord_id == guild_id) {
            let user_details = state
                .discord
                .get_member(guild_id.parse()?, discord_user_id.into())
                .await?;

            let record = guild_access::ActiveModel {
                user_id: Set(user_id),
                guild_id: Set(guild_record.id),
                is_owner: Set(guild.owner),
                is_administrator: Set(user_details
                    .permissions
                    .map(|p| p.administrator())
                    .unwrap_or(false)),
                roles: Set(RoleInfo {
                    role_ids: user_details
                        .roles
                        .iter()
                        .map(|r| Into::<u64>::into(*r))
                        .collect(),
                }),
            };
            guild_access::Entity::insert(record)
                .on_conflict(
                    OnConflict::columns([
                        guild_access::Column::GuildId,
                        guild_access::Column::UserId,
                    ])
                    .update_columns([
                        guild_access::Column::IsOwner,
                        guild_access::Column::IsAdministrator,
                        guild_access::Column::Roles,
                    ])
                    .to_owned(),
                )
                .exec(state.db.as_ref())
                .await?;
        }
    }

    Ok(Redirect::to("/dashboard"))
}

fn oauth_client(config: &Config) -> Result<BasicClient, anyhow::Error> {
    Ok(BasicClient::new(
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string())?,
        Some(TokenUrl::new(
            "https://discord.com/api/oauth2/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?))
}
