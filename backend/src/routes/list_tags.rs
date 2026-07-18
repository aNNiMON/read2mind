use std::collections::HashMap;

use axum::{Json, extract::State};

use crate::{AppState, db_index, error::AppError};

pub async fn handler(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, usize>>, AppError> {
    let tags = db_index::load_tags_by_freq(&state.db)?;
    Ok(Json(tags))
}
