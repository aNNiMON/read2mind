use std::collections::HashMap;

use axum::{Json, extract::State};
use chrono::SecondsFormat;
use serde::Deserialize;
use tracing::debug;

use crate::{
    AppState, db_index,
    error::AppError,
    jobs::{download_job::DownloadJob, fetch_job::FetchJob},
    model::{
        attachment::BANNER_FILE_NAME,
        item::{Item, ItemKind, ItemMetadata, ItemStatus},
    },
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
    let tags = validate::validate_tags(req.tags.as_deref())?;

    let mut m = ItemMetadata {
        kind: req.kind,
        title: "Untitled".to_owned(),
        url: req.url.clone(),
        tags,
        status: ItemStatus::New,
        created_at: created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        ..Default::default()
    };

    let mut downloads: HashMap<&str, DownloadJob> = HashMap::new();
    let content = match req.kind {
        ItemKind::Article => {
            // Fetch title, author and markdown content
            let fetch_job = FetchJob::new(req.url.unwrap(), req.content.clone());
            let result = fetch_job.run().map_err(AppError::FetchError)?;
            debug!(?result, "create_item Article");
            m.title = req.title.or(result.title).unwrap_or(m.title);
            m.author = req.author.or(result.author);
            m.published_at = result.published;
            if let Some(image) = result.image {
                downloads.insert(BANNER_FILE_NAME, DownloadJob::new(state.client, image));
            }
            Some(result.content)
        }
        ItemKind::Video => {
            // Fetch title, author and transcript
            let fetch_job = FetchJob::from_url(req.url.unwrap());
            let result = fetch_job.run().map_err(AppError::FetchError)?;
            debug!(?result, "create_item Video");
            m.title = req.title.or(result.title).unwrap_or(m.title);
            m.author = req.author.or(result.author);
            m.published_at = result.published;
            if let Some(image) = result.image {
                downloads.insert(BANNER_FILE_NAME, DownloadJob::new(state.client, image));
            }
            req.content.or(Some(result.content))
        }
        ItemKind::Note => {
            m.title = req.title.unwrap_or(m.title);
            req.content
        }
        ItemKind::Task => {
            m.title = req.title.unwrap_or(m.title);
            req.content
        }
    };

    // Save metadata.json
    let dir_path = state.storage.item_full_path(&created_at, &m.title);
    state.storage.save_metadata_by_dir(&m, &dir_path)?;

    // Save file with item content
    if let Some(content) = content {
        state.storage.save_content(&content, &dir_path)?;
    }

    // Download in background
    for (filename, job) in downloads {
        let storage = state.storage.clone();
        let dir_path = dir_path.clone();
        tokio::spawn(async move {
            if let Ok(cursor) = job.run().await {
                let _ = storage.copy_from_cursor(cursor, &dir_path, filename);
            }
        });
    }

    // Create item and update index
    let path = state.storage.item_path(&created_at, &m.title);
    let item = Item::from_metadata(m, path);
    db_index::add_item(&state.db, &item)?;

    Ok(Json(item))
}
