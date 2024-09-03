use std::{num::ParseIntError, str::FromStr};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{basic::BasicErrorResponseType, RequestTokenError, StandardErrorResponse};

pub enum Error {
    Anyhow(anyhow::Error),
    LoggedOut,
    Forbidden,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Anyhow(err) => {
                let body = format!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Self::LoggedOut => Redirect::to("/").into_response(),
            Self::Forbidden => (StatusCode::FORBIDDEN, "".to_string()).into_response(),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<tower_sessions::session::Error> for Error {
    fn from(value: tower_sessions::session::Error) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

// Yes I know I should be able to do this with generics. TODO.
impl
    From<
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    > for Error
{
    fn from(
        value: RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}
