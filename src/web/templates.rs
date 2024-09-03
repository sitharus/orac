use std::collections::HashMap;

use super::filters;
use crate::entities::*;
use askama::Template;

pub struct GuildItem {
    pub name: String,
    pub id: i32,
    pub logo_url: Option<String>,
    pub selected: bool,
}

pub struct Common {
    pub page_title: String,
    pub guilds: Vec<GuildItem>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub username: Option<String>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct Dashboard {
    pub common: Common,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile {
    pub common: Common,
    pub name: String,
    pub email: String,
}

#[derive(Template)]
#[template(path = "guild.html")]
pub struct GuildPage {
    pub common: Common,
    pub guild: guild::Model,
    pub managed_channels: Vec<channel::Model>,
}

#[derive(Template)]
#[template(path = "channels.html")]
pub struct Channels {
    pub common: Common,
    pub channels: Vec<channel::Model>,
    pub guild_id: i32,
}

#[derive(Template)]
#[template(path = "add_channel.html")]
pub struct AddChannel {
    pub common: Common,
    pub channels: Vec<serenity::model::channel::GuildChannel>,
    pub guild_id: i32,
}
