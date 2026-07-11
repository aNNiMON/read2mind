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
    routes::request_util::{filter_non_blank, get_non_empty_title},
    validate,
};

/// Request to update item metadata. Performs rename in storage directory.
#[derive(Debug, Deserialize)]
pub struct UpdateItemRequest {
    pub title: String,
    pub created_at: String,
    pub author: Option<String>,
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
        author: filter_non_blank(req.author),
        ..metadata
    };
    let item = if path == new_path {
        state.storage.save_metadata(&new_metadata, &path)?;
        // Update index
        let item = Item::from_metadata(new_metadata, new_path);
        db_index::add_item(&state.db, &item)?;
        item
    } else {
        state.storage.rename_item(&path, &new_path)?;
        state.storage.save_metadata(&new_metadata, &new_path)?;
        // Update index
        let item = Item::from_metadata(new_metadata, new_path);
        db_index::delete_item(&state.db, &path)?;
        db_index::add_item(&state.db, &item)?;
        item
    };
    Ok(Json(item))
}
