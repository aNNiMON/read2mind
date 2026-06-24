use axum::{Json, extract::State};
use chrono::SecondsFormat;
use serde::Deserialize;

use crate::{
    AppState,
    error::AppError,
    item::{Item, ItemKind, ItemStatus},
    validate,
};

/// Request to create a standalone note/task (or any item) without fetching.
#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    #[serde(default)]
    pub kind: ItemKind,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

pub async fn handler(
    State(_state): State<AppState>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Json<Item>, AppError> {
    validate::validate_item(
        req.kind,
        req.url.as_ref(),
        req.title.as_ref(),
        req.content.as_ref(),
    )?;
    let created_at = validate::validate_datetime(req.created_at)?;
    let tags = validate::validate_tags(req.tags)?;

    Ok(Json(Item {
        kind: req.kind,
        path: "/todo".to_string(),
        title: req.title.unwrap_or_default(),
        url: req.url,
        tags,
        status: ItemStatus::New,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        updated_at: None,
    }))
}
