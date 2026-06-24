use chrono::{DateTime, Local};

use crate::{error::AppError, item::ItemKind};

/// Validates item fields according to item kind
pub fn validate_item(
    kind: ItemKind,
    url: Option<&String>,
    title: Option<&String>,
    content: Option<&String>,
) -> Result<(), AppError> {
    match kind {
        ItemKind::Article => {
            if url.is_none() {
                return Err(AppError::InvalidRequest("URL is empty".to_owned()));
            }
        }
        ItemKind::Bookmark => {
            if url.is_none() {
                return Err(AppError::InvalidRequest("URL is empty".to_owned()));
            }
        }
        ItemKind::Note => {
            if title.is_none() {
                return Err(AppError::InvalidRequest("Title is empty".to_owned()));
            }
            if content.is_none() {
                return Err(AppError::InvalidRequest("Content is empty".to_owned()));
            }
        }
        ItemKind::Task => {
            if title.is_none() {
                return Err(AppError::InvalidRequest("Title is empty".to_owned()));
            }
            if content.is_none() {
                return Err(AppError::InvalidRequest("Content is empty".to_owned()));
            }
        }
        ItemKind::Video => {
            if url.is_none() {
                return Err(AppError::InvalidRequest("URL is empty".to_owned()));
            }
        }
    }
    Ok(())
}

/// Validates and returns datetime in RFC 3339 format
pub fn validate_datetime(datetime: Option<String>) -> Result<DateTime<Local>, AppError> {
    let dt = match datetime {
        Some(dt_str) => chrono::DateTime::parse_from_rfc3339(&dt_str)
            .map_err(|e| {
                AppError::InvalidRequest(format!("Invalid created_at (must be RFC 3339): {e}"))
            })?
            .with_timezone(&chrono::Local),
        None => chrono::Local::now(),
    };
    Ok(dt)
}

/// Validate tags string and return vector of tags
pub fn validate_tags(tags: Option<String>) -> Result<Vec<String>, AppError> {
    let tags: Vec<String> = tags
        .as_ref()
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Ok(tags)
}
