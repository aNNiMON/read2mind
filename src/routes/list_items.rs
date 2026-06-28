use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    AppState,
    db_index::{self, ItemsFilter},
    error::AppError,
    item::Item,
};

#[derive(Debug, Deserialize, Default)]
pub struct ListItemsRequest {
    pub kind: Option<String>,
    pub status: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl From<ListItemsRequest> for ItemsFilter {
    fn from(value: ListItemsRequest) -> Self {
        Self {
            kind: value.kind,
            status: value.status,
            limit: value.limit.unwrap_or(50),
            offset: value.offset.unwrap_or_default(),
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Query(query): Query<ListItemsRequest>,
) -> Result<Json<Vec<Item>>, AppError> {
    let items = db_index::load_items(&state.db, &query.into())?;
    Ok(Json(items))
}
