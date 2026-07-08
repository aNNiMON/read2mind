use axum::{Json, extract::State};

use crate::{AppState, db_index, error::AppError};

pub async fn handler(State(state): State<AppState>) -> Result<Json<Vec<String>>, AppError> {
    let tags = db_index::load_all_tags(&state.db)?;
    Ok(Json(tags))
}
