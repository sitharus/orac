use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
