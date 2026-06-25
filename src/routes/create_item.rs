use axum::{Json, extract::State};
use chrono::SecondsFormat;
use serde::Deserialize;

use crate::{
    AppState,
    error::AppError,
    item::{Item, ItemKind, ItemStatus},
    jobs::fetch_job::FetchJob,
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
    pub author: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
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

    let mut item = Item {
        kind: req.kind,
        title: "Untitled".to_owned(),
        url: req.url.clone(),
        tags,
        status: ItemStatus::New,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        ..Default::default()
    };

    match req.kind {
        ItemKind::Article => {
            // Fetch title, author and markdown content
            let fetch_job = FetchJob::new(req.url.unwrap(), req.content);
            let result = fetch_job.run().map_err(AppError::FetchError)?;
            item.title = req.title.or(result.title).unwrap_or(item.title);
            item.author = req.author.or(result.author);
        }
        ItemKind::Video => {
            // Fetch title, author and transcript
            let fetch_job = FetchJob::from_url(req.url.unwrap());
            let result = fetch_job.run().map_err(AppError::FetchError)?;
            item.title = req.title.or(result.title).unwrap_or(item.title);
            item.author = req.author.or(result.author);
        }
        ItemKind::Bookmark => {
            // Fetch title
            let fetch_job = FetchJob::from_url(req.url.unwrap());
            let result = fetch_job.run().map_err(AppError::FetchError)?;
            item.title = req.title.or(result.title).unwrap_or(item.title);
        }
        ItemKind::Note => {
            item.title = req.title.unwrap_or(item.title);
        }
        ItemKind::Task => {
            item.title = req.title.unwrap_or(item.title);
        }
    };
    item.path = state.storage.item_path(&created_at, &item.title);

    Ok(Json(item))
}
