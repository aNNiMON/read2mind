use axum::{Json, extract::State};

use crate::{AppState, config::SavedSearch};

pub async fn handler(State(state): State<AppState>) -> Json<Vec<SavedSearch>> {
    Json(state.config.saved_search.clone())
}
