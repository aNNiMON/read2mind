use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{AppState, error::AppError, item::Item};

#[derive(Debug, Deserialize, Default)]
pub struct ListItemsRequest {}

pub async fn handler(
    State(state): State<AppState>,
    Query(_query): Query<ListItemsRequest>,
) -> Result<Json<Vec<Item>>, AppError> {
    let items = state.storage.list_items()?;
    Ok(Json(items))
}
