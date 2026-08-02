use std::path::{Component, Path};

use chrono::{DateTime, Local};
use reqwest::Url;

use crate::{error::AppError, model::attachment, model::item::ItemKind};

const BYTES_IN_MIB: usize = 1024 * 1024;

const MAX_ATTACHMENT_MB: usize = 50;
const MAX_ATTACHMENT_SIZE: usize = MAX_ATTACHMENT_MB * BYTES_IN_MIB;
const MAX_CONTENT_MB: usize = 4;
pub const MAX_CONTENT_LEN: usize = MAX_CONTENT_MB * BYTES_IN_MIB;
pub const MAX_BODY_SIZE: usize = MAX_ATTACHMENT_SIZE + 2 * BYTES_IN_MIB;

/// Validates item fields according to item kind
pub fn validate_item(
    kind: ItemKind,
    url: Option<&String>,
    content: Option<&String>,
) -> Result<(), AppError> {
    if let Some(content) = content {
        if content.len() > MAX_CONTENT_LEN {
            return Err(AppError::InvalidRequest(format!(
                "Content exceeds maximum length of {} MiB",
                MAX_CONTENT_MB
            )));
        }
    }

    match kind {
        ItemKind::Article => {
            if url.is_none() {
                return Err(AppError::InvalidRequest("URL is empty".to_owned()));
            }
        }
        ItemKind::Note => {
            if content.is_none() {
                return Err(AppError::InvalidRequest("Content is empty".to_owned()));
            }
        }
        ItemKind::Task => {
            if content.is_none() {
                return Err(AppError::InvalidRequest("Content is empty".to_owned()));
            }
        }
        ItemKind::Video => {
            if let Some(url) = url {
                let parsed_url = url
                    .parse::<Url>()
                    .map_err(|e| AppError::InvalidRequest(format!("Invalid URL: {e}")))?;
                let domain = parsed_url
                    .domain()
                    .ok_or_else(|| AppError::InvalidRequest("Invalid URL".to_owned()))?;
                if !matches!(
                    domain,
                    "www.youtube.com" | "youtube.com" | "youtu.be" | "m.youtube.com"
                ) {
                    return Err(AppError::InvalidRequest(
                        "Only YouTube URLs are allowed".to_owned(),
                    ));
                }
            } else {
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
pub fn validate_tags(tags: Option<&str>) -> Result<Vec<String>, AppError> {
    let tags: Vec<String> = tags
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Ok(tags)
}

/// Validates filename of an attachment
pub fn validate_attachment_filename(filename: &str) -> Result<&str, AppError> {
    let filename = filename.trim();
    if filename.is_empty() {
        return Err(AppError::InvalidRequest(
            "filename cannot be empty".to_owned(),
        ));
    }

    let path = Path::new(filename);
    if is_invalid_path(path) {
        return Err(AppError::InvalidRequest(format!(
            "Invalid filename: {}",
            filename
        )));
    }

    if path
        .components()
        .any(|component| component.as_os_str() == attachment::METADATA_FILE_NAME)
    {
        return Err(AppError::InvalidRequest(format!(
            "Reserved filename {}",
            filename
        )));
    }

    Ok(filename)
}

fn is_invalid_path(path: &Path) -> bool {
    path.is_absolute()
        || path.components().count() != 1
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
}

/// Validates an item path identifier generated as `YYYYMMDD <title>`.
pub fn validate_item_path(item_path: &str) -> Result<&str, AppError> {
    if item_path.is_empty() {
        return Err(AppError::InvalidRequest(
            "item path cannot be empty".to_owned(),
        ));
    }

    let path = Path::new(item_path);
    if is_invalid_path(path) || item_path.contains('\\') {
        return Err(AppError::InvalidRequest(format!(
            "Invalid item path: {}",
            item_path
        )));
    }

    let bytes = item_path.as_bytes();
    if bytes.len() < 9 || bytes[8] != b' ' || !bytes[..8].iter().all(u8::is_ascii_digit) {
        return Err(AppError::InvalidRequest(format!(
            "Invalid item path: {}",
            item_path
        )));
    }

    Ok(item_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_item_path_correct() {
        assert_eq!(
            validate_item_path("20260705 Example item").unwrap(),
            "20260705 Example item"
        );
    }

    #[test]
    fn validate_item_path_invalid() {
        let cases = vec![
            "../../../../etc/passwd",
            "20260705 ../../etc/passwd",
            "20260705 ..\\..\\etc\\passwd",
            "",
            "/20260705 Example item",
            "20260705",
            "2026 Example item",
            "20260705Example item",
        ];
        for case in cases {
            assert!(validate_item_path(case).is_err());
        }
    }
}
