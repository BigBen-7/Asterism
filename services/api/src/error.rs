use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid schema document: {0}")]
    InvalidSchema(String),
    #[error("invalid credential: {0}")]
    InvalidCredential(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("invalid verification payload: {0}")]
    InvalidPayload(String),
    #[error("could not read chain data: {0}")]
    Chain(String),
    #[error("could not generate a random salt")]
    Random,
    #[error("missing environment variable {0}")]
    MissingEnv(&'static str),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::InvalidSchema(_)
            | Error::InvalidCredential(_)
            | Error::InvalidRequest(_)
            | Error::InvalidPayload(_) => StatusCode::BAD_REQUEST,
            Error::Chain(_) | Error::Database(_) => StatusCode::BAD_GATEWAY,
            Error::Random | Error::MissingEnv(_) | Error::Io(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
