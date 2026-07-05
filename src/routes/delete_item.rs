use axum::extract::{Path, State};
use reqwest::StatusCode;

use crate::{AppState, db_index, error::AppError};

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<StatusCode, AppError> {
    state.storage.delete_item(&path)?;
    db_index::delete_item(&state.db, &path)?;
    Ok(StatusCode::OK)
}
