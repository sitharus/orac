use std::collections::HashMap;
use std::sync::Arc;

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
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::appstate::AppState;
use super::errors;

#[derive(Serialize, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    global_name: String,
    email: String,
}

pub async fn start_oauth(State(state): State<Arc<AppState>>) -> Result<Redirect, errors::Error> {
    // Generate a PKCE challenge.

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = oauth_client(&state.config)?
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("guilds".to_string()))
        .add_scope(Scope::new("identify".to_string()))
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
    println!("access token {:?}", access_token);

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
    session.insert("OAUTH_TOKEN", unwrapped).await?;
    match user {
        Some(u) => {
            session.insert(USER_ID_KEY, u.id).await?;
            Ok(Redirect::to("/oauth/select_guild"))
        }
        None => Ok(Redirect::to("/")),
    }
}

pub async fn select_guild(session: Session) -> Result<Redirect, errors::Error> {
    let oauth_token = session
        .get::<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>("OAUTH_TOKEN")
        .await?;

    Err(anyhow::anyhow!("TODO").into())
}

fn oauth_client(config: &Config) -> Result<BasicClient, anyhow::Error> {
    Ok(BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string())?,
        Some(TokenUrl::new(
            "https://discord.com/api/oauth2/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url)?))
}
