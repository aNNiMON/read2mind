use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    InvalidRequest(String),
    FetchError(String),
    NotFound(String),
    FsError(String),
    DbError(String),
    ConfigError(String),
    AiError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, msg) = match self {
            AppError::InvalidRequest(m) => ("InvalidRequest", m),
            AppError::FetchError(m) => ("FetchError", m),
            AppError::NotFound(m) => ("NotFound", m),
            AppError::FsError(m) => ("FsError", m),
            AppError::DbError(m) => ("DbError", m),
            AppError::ConfigError(m) => ("ConfigError", m),
            AppError::AiError(m) => ("AiError", m),
        };
        write!(f, "{kind}: {msg}")
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::FetchError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::FsError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::DbError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::AiError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        let body = json!({ "error": error_message });
        (status, Json(body)).into_response()
    }
}
