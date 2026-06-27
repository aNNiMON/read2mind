use std::{fs, path::PathBuf};

use chrono::{DateTime, Datelike, Local};
use glob::glob;

use crate::{
    error::AppError,
    item::{Item, ItemMetadata},
};

const MAX_FOLDER_LENGTH: usize = 120;
const METADATA_FILE_NAME: &str = "metadata.json";
const CONTENT_FILE_NAME: &str = "content.md";

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

    pub fn list_items(&self) -> Result<Vec<Item>, AppError> {
        // data/2026/20260601 Name/metadata.json
        let pattern = format!(
            "{}/20[2-5][0-9]/20[2-5][0-9][01][0-9][0-3][0-9] */{}",
            self.data_dir.display(),
            METADATA_FILE_NAME
        );
        let files = glob(&pattern)
            .map_err(|e| AppError::FsError(format!("Failed to list files: {}", e)))?;
        let mut items = Vec::new();
        for file in files {
            let path =
                file.map_err(|e| AppError::FsError(format!("Failed to get file path: {}", e)))?;
            let item_path = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|p| p.to_str())
                .ok_or_else(|| AppError::FsError("Failed to get parent directory".into()))?;
            let metadata = self.read_metadata(item_path)?;
            let item = Item::from_metadata(metadata, item_path.to_string());
            items.push(item);
        }
        Ok(items)
    }

    /// Read and parse metadata.json file
    pub fn read_metadata(&self, item_path: &str) -> Result<ItemMetadata, AppError> {
        let year = item_path.chars().take(4).collect::<String>();
        let dir = self.data_dir.join(year).join(item_path);
        let metadata_path = dir.join(METADATA_FILE_NAME);
        let metadata_json = fs::read_to_string(&metadata_path).map_err(|e| {
            AppError::FsError(format!("Failed to read {}: {}", METADATA_FILE_NAME, e))
        })?;
        let metadata: ItemMetadata = serde_json::from_str(&metadata_json).map_err(|e| {
            AppError::FsError(format!("Failed to parse {}: {}", METADATA_FILE_NAME, e))
        })?;
        Ok(metadata)
    }

    /// Save metadata.json file
    pub fn save_metadata(&self, metadata: &ItemMetadata, dir: &PathBuf) -> Result<(), AppError> {
        Self::create_dir(dir)?;
        let metadata_path = dir.join(METADATA_FILE_NAME);
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| AppError::FsError(format!("Failed to serialize metadata JSON: {}", e)))?;
        fs::write(&metadata_path, metadata_json).map_err(|e| {
            AppError::FsError(format!("Failed to write {}: {}", METADATA_FILE_NAME, e))
        })?;
        Ok(())
    }

    /// Save content.md file
    pub fn save_content(&self, content: &str, dir: &PathBuf) -> Result<(), AppError> {
        Self::create_dir(dir)?;
        let content_path = dir.join(CONTENT_FILE_NAME);
        fs::write(&content_path, content).map_err(|e| {
            AppError::FsError(format!("Failed to write {}: {}", CONTENT_FILE_NAME, e))
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
        sanitized.trim().to_owned()
    }

    fn create_dir(dir: &PathBuf) -> Result<(), AppError> {
        fs::create_dir_all(dir)
            .map_err(|e| AppError::FsError(format!("Failed to create directory {:?}: {}", dir, e)))
    }
}
