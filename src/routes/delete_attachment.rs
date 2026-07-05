use axum::extract::{Path, State};
use chrono::{Local, SecondsFormat};
use reqwest::StatusCode;

use crate::{AppState, db_index, error::AppError, item::Item};

pub async fn handler(
    State(state): State<AppState>,
    Path((path, name)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let mut metadata = state.storage.read_metadata(&path)?;
    state.storage.delete_attachment(&path, &name)?;

    // Update index
    metadata.updated_at = Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    state.storage.save_metadata(&metadata, &path)?;
    let item = Item::from_metadata(metadata, path);
    db_index::add_item(&state.db, &item)?;
    Ok(StatusCode::OK)
}
