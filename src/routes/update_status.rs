use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{Local, SecondsFormat};
use serde::Deserialize;

use crate::{
    AppState, db_index,
    error::AppError,
    model::item::{Item, ItemStatus},
};

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: ItemStatus,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<Json<Item>, AppError> {
    let mut metadata = state.storage.read_metadata(&path)?;
    if metadata.status != req.status {
        metadata.status = req.status;
        metadata.updated_at = Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        state.storage.save_metadata(&metadata, &path)?;
    }

    let item = Item::from_metadata(metadata, path);
    db_index::add_item(&state.db, &item)?;
    Ok(Json(item))
}
