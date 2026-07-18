use reqwest::StatusCode;

use crate::error::AppError;

pub async fn handler() -> Result<StatusCode, AppError> {
    Ok(StatusCode::OK)
}
