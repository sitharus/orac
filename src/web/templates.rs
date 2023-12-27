use super::filters;
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
