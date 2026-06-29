use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{Local, SecondsFormat};
use serde::Deserialize;

use crate::{AppState, db_index, error::AppError, item::Item, validate};

#[derive(Debug, Deserialize)]
pub struct UpdateTagsRequest {
    pub tags: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(req): Json<UpdateTagsRequest>,
) -> Result<Json<Item>, AppError> {
    let tags = validate::validate_tags(Some(&req.tags))?;

    let mut metadata = state.storage.read_metadata(&path)?;
    metadata.tags = tags;
    metadata.updated_at = Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    state.storage.save_metadata(&metadata, &path)?;

    let item = Item::from_metadata(metadata, path);
    db_index::add_item(&state.db, &item)?;
    Ok(Json(item))
}
