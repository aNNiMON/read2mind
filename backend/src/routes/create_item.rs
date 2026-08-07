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
    routes::request_util::{filter_non_blank, get_non_empty_title},
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
    pub cover_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(req): Json<CreateItemRequest>,
) -> Result<Json<Item>, AppError> {
    validate::validate_item(req.kind, req.url.as_ref(), req.content.as_ref())?;
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
        ItemKind::Article | ItemKind::Video => {
            let skip_fetch = req.title.is_some() && req.content.is_some();
            let result = if skip_fetch {
                // Skip fetching
                FetchResult {
                    content: req.content.unwrap(),
                    title: get_non_empty_title(req.title, None),
                    author: req.author,
                    published: req.published_at,
                    image: req.cover_url,
                }
            } else {
                // Fetch title, author and markdown content / transcript
                let fetch_job = FetchJob::new(req.url.unwrap());
                let result = fetch_job.run().map_err(AppError::FetchError)?;
                debug!(?result, "create_item decruft result {:?}", req.kind);
                FetchResult {
                    content: result.content,
                    title: get_non_empty_title(req.title, result.title),
                    author: filter_non_blank(req.author)
                        .or_else(|| filter_non_blank(result.author)),
                    published: filter_non_blank(req.published_at)
                        .or_else(|| filter_non_blank(result.published)),
                    image: filter_non_blank(req.cover_url)
                        .or_else(|| filter_non_blank(result.image)),
                }
            };

            debug!(?result, "create_item {:?}", req.kind);
            m.title = result.title;
            m.author = result.author;
            m.published_at = result.published;
            if let Some(image) = result.image {
                downloads.insert(BANNER_FILE_NAME, DownloadJob::new(state.client, image));
            }
            Some(result.content)
        }
        ItemKind::Note => {
            m.title = get_non_empty_title(req.title, None);
            req.content
        }
        ItemKind::Task => {
            m.title = get_non_empty_title(req.title, None);
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

#[derive(Debug)]
struct FetchResult {
    title: String,
    content: String,
    author: Option<String>,
    published: Option<String>,
    image: Option<String>,
}
