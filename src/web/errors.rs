use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use oauth2::{basic::BasicErrorResponseType, RequestTokenError, StandardErrorResponse};

pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = format!("{:?}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl From<tower_sessions::session::Error> for Error {
    fn from(value: tower_sessions::session::Error) -> Self {
        Self(anyhow::anyhow!(value))
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
        Self(anyhow::anyhow!(value))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self(anyhow::anyhow!(value))
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(value: sea_orm::DbErr) -> Self {
        Self(anyhow::anyhow!(value))
    }
}
