use axum::{
    Json,
    extract::{Path, State},
};

use crate::{AppState, error::AppError, model::item::Item};

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<Item>, AppError> {
    let metadata = state.storage.read_metadata(&path)?;
    Ok(Json(Item::from_metadata(metadata, path)))
}
