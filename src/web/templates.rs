use std::collections::HashMap;

use super::filters;
use crate::entities::*;
use askama::Template;

pub struct Common<'a> {
    pub page_title: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub username: Option<String>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct Dashboard<'a> {
    pub common: Common<'a>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile<'a> {
    pub common: Common<'a>,
    pub name: String,
    pub email: String,
    pub discord_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "guilds.html")]
pub struct Guilds<'a> {
    pub common: Common<'a>,
    pub guilds: Vec<guild::Model>,
}

#[derive(Template)]
#[template(path = "channels.html")]
pub struct Channels<'a> {
    pub common: Common<'a>,
    pub channels: Vec<channel::Model>,
    pub guilds: Vec<guild::Model>,
    pub guild_map: HashMap<i32, guild::Model>,
}
