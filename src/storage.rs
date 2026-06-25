use std::{fs, path::PathBuf};

use chrono::{DateTime, Datelike, Local};

use crate::{error::AppError, item::ItemMetadata};

const MAX_FOLDER_LENGTH: usize = 120;
const METADATA_FILE_NAME: &str = "metadata.json";

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Build the item's directory name `YYYYMMDD <sanitized title>`
    pub fn item_path(&self, dt: &DateTime<Local>, title: &str) -> String {
        let sanitized = Self::sanitize_title(title);
        format!("{} {}", dt.format("%Y%m%d"), &sanitized)
    }

    /// Build the item's full path /data/<year>/<YYYYMMDD <sanitized title>>
    pub fn item_full_path(&self, dt: &DateTime<Local>, title: &str) -> PathBuf {
        let year = dt.year();
        let item_dir = self.item_path(dt, title);
        self.data_dir.join(year.to_string()).join(item_dir)
    }

    pub fn save(&self, metadata: &ItemMetadata, item_dir_path: PathBuf) -> Result<(), AppError> {
        fs::create_dir_all(&item_dir_path).map_err(|e| {
            AppError::FsError(format!(
                "Failed to create directory {:?}: {}",
                item_dir_path, e
            ))
        })?;
        let metadata_path = item_dir_path.join(METADATA_FILE_NAME);
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| AppError::FsError(format!("Failed to serialize metadata JSON: {}", e)))?;
        fs::write(&metadata_path, metadata_json).map_err(|e| {
            AppError::FsError(format!("Failed to write {}: {}", METADATA_FILE_NAME, e))
        })?;
        Ok(())
    }

    /// Sanitize title for folder name, limiting to 120 characters
    fn sanitize_title(title: &str) -> String {
        let sanitized: String = title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .take(MAX_FOLDER_LENGTH)
            .collect();
        sanitized.trim().to_string()
    }
}
