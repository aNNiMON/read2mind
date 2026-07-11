use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{Local, SecondsFormat};
use serde::Deserialize;

use crate::{
    AppState, db_index,
    error::AppError,
    model::item::{Item, ItemMetadata},
    routes::request_util::get_non_empty_title,
    validate,
};

/// Request to update item metadata. Performs rename in storage directory.
#[derive(Debug, Deserialize)]
pub struct UpdateItemRequest {
    pub title: String,
    pub created_at: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<Item>, AppError> {
    let metadata = state.storage.read_metadata(&path)?;

    let created_at = validate::validate_datetime(Some(req.created_at))?;
    let title = get_non_empty_title(Some(req.title), Some(metadata.title));

    // determine if new path is different from old path
    let new_path = state.storage.item_path(&created_at, &title);
    let new_metadata = ItemMetadata {
        title,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        updated_at: Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
        ..metadata
    };
    if path == new_path {
        state.storage.save_metadata(&new_metadata, &path)?;
        // Update index
        let item = Item::from_metadata(new_metadata, new_path);
        db_index::add_item(&state.db, &item)?;
        Ok(Json(item))
    } else {
        state.storage.rename_item(&path, &new_path)?;
        state.storage.save_metadata(&new_metadata, &new_path)?;
        // Update index
        let item = Item::from_metadata(new_metadata, new_path);
        db_index::delete_item(&state.db, &path)?;
        db_index::add_item(&state.db, &item)?;
        Ok(Json(item))
    }
}
