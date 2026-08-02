use axum::http::StatusCode;

pub async fn handler() -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}
