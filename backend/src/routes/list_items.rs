use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db_index::{self, ItemsFilter},
    error::AppError,
    model::item::Item,
};

#[derive(Debug, Deserialize, Default)]
pub struct ListItemsRequest {
    pub kind: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Default)]
pub struct ListItemsResponse {
    pub items: Vec<Item>,
    pub total: usize,
}

impl From<ListItemsRequest> for ItemsFilter {
    fn from(value: ListItemsRequest) -> Self {
        let (include_tags, exclude_tags) = parse_tags_filter(value.tags.as_deref());
        Self {
            kind: value.kind,
            status: value.status,
            date: value.date,
            keyword: value.keyword,
            author: value.author,
            include_tags,
            exclude_tags,
            limit: value.limit.unwrap_or(50),
            offset: value.offset.unwrap_or_default(),
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Query(query): Query<ListItemsRequest>,
) -> Result<Json<ListItemsResponse>, AppError> {
    let (items, total) = db_index::load_items(&state.db, &query.into())?;
    Ok(Json(ListItemsResponse { items, total }))
}

fn parse_tags_filter(tags: Option<&str>) -> (Vec<String>, Vec<String>) {
    let mut include = Vec::new();
    let mut exclude = Vec::new();
    for raw_tag in tags.unwrap_or_default().split(',') {
        let tag = raw_tag.trim();
        if tag.is_empty() {
            continue;
        }
        if let Some(excluded_tag) = tag.strip_prefix('!') {
            let excluded_tag = excluded_tag.trim();
            if !excluded_tag.is_empty() {
                exclude.push(excluded_tag.to_owned());
            }
        } else {
            include.push(tag.to_owned());
        }
    }
    (include, exclude)
}
