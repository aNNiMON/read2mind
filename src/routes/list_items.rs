use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{AppState, db_index, error::AppError, item::Item};

#[derive(Debug, Deserialize, Default)]
pub struct ListItemsRequest {}

pub async fn handler(
    State(state): State<AppState>,
    Query(_query): Query<ListItemsRequest>,
) -> Result<Json<Vec<Item>>, AppError> {
    let items = db_index::load_items(&state.db)?;
    Ok(Json(items))
}
