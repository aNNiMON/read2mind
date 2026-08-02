use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::{AppState, error::AppError};

pub async fn require_api_secret(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_secret = state
        .config
        .api_secret
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("API secret is not set".to_owned()))?;

    let hashed_token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if hashed_token != Some(api_secret.as_str()) {
        return Err(AppError::Unauthorized("Unauthorized".to_owned()));
    }

    Ok(next.run(request).await)
}
