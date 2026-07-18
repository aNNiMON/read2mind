use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Cursor, Write},
    path::PathBuf,
};

use axum::body::Bytes;
use chrono::{DateTime, Datelike, Local};
use glob::glob;

use crate::{
    error::AppError,
    model::{
        attachment::{BANNER_FILE_NAME, CONTENT_FILE_NAME, METADATA_FILE_NAME},
        item::{Item, ItemMetadata},
    },
    validate,
};

const MAX_FOLDER_LENGTH: usize = 120;

pub struct Storage {
    data_dir: PathBuf,
}

#[derive(Debug)]
pub struct AttachmentsList {
    pub path: String,
    pub attachments: HashSet<String>,
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
        let files =
            glob(&pattern).map_err(|e| AppError::FsError(format!("Failed to list files: {e}")))?;
        let mut items = Vec::new();
        for file in files {
            let path =
                file.map_err(|e| AppError::FsError(format!("Failed to get file path: {e}")))?;
            let item_path = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|p| p.to_str())
                .ok_or_else(|| AppError::FsError("Failed to get parent directory".into()))?;
            let metadata = self.read_metadata(item_path)?;
            let item = Item::from_metadata(metadata, item_path.to_owned());
            items.push(item);
        }
        Ok(items)
    }

    /// Read and parse metadata.json file
    pub fn read_metadata(&self, item_path: &str) -> Result<ItemMetadata, AppError> {
        let metadata_path = self.item_dir(item_path)?.join(METADATA_FILE_NAME);
        if !metadata_path.exists() {
            return Err(AppError::NotFound(format!("Item not found: {item_path}")));
        }
        let metadata_json = fs::read_to_string(&metadata_path)
            .map_err(|e| AppError::FsError(format!("Failed to read {METADATA_FILE_NAME}: {e}")))?;
        let metadata: ItemMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| AppError::FsError(format!("Failed to parse {METADATA_FILE_NAME}: {e}")))?;
        Ok(metadata)
    }

    /// Save metadata.json file
    pub fn save_metadata(&self, metadata: &ItemMetadata, item_path: &str) -> Result<(), AppError> {
        let dir = self.item_dir(item_path)?;
        self.save_metadata_by_dir(metadata, &dir)
    }

    /// Save metadata.json file by date and title
    pub fn save_metadata_by_date_and_title(
        &self,
        metadata: &ItemMetadata,
        dt: &DateTime<Local>,
        title: &str,
    ) -> Result<(), AppError> {
        let dir = self.item_full_path(dt, title);
        self.save_metadata_by_dir(metadata, &dir)
    }

    /// Save metadata.json file by directory
    pub fn save_metadata_by_dir(
        &self,
        metadata: &ItemMetadata,
        dir: &PathBuf,
    ) -> Result<(), AppError> {
        Self::create_dir(dir)?;
        let metadata_path = dir.join(METADATA_FILE_NAME);
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| AppError::FsError(format!("Failed to serialize metadata JSON: {e}")))?;
        fs::write(&metadata_path, metadata_json)
            .map_err(|e| AppError::FsError(format!("Failed to write {METADATA_FILE_NAME}: {e}")))?;
        Ok(())
    }

    /// Rename item directory
    pub fn rename_item(&self, old_path: &str, new_path: &str) -> Result<(), AppError> {
        let old_dir = self.item_dir(old_path)?;
        if !old_dir.exists() {
            return Err(AppError::NotFound(format!("Item not found: {old_path}")));
        }
        let new_dir = self.item_dir(new_path)?;
        if new_dir.exists() {
            return Err(AppError::FsError(format!(
                "Item already exists: {new_dir:?}"
            )));
        }
        Self::create_dir(&new_dir)?;
        fs::rename(&old_dir, &new_dir)
            .map_err(|e| AppError::FsError(format!("Failed to move item: {e}")))?;
        Ok(())
    }

    /// Delete item directory
    pub fn delete_item(&self, item_path: &str) -> Result<(), AppError> {
        let dir = self.item_dir(item_path)?;
        if !dir.exists() {
            return Err(AppError::NotFound(format!("Item not found: {item_path}")));
        }
        fs::remove_dir_all(&dir)
            .map_err(|e| AppError::FsError(format!("Failed to delete item: {e}")))?;
        Ok(())
    }

    pub fn list_attachments(&self, item_path: &str) -> Result<AttachmentsList, AppError> {
        let dir = self.item_dir(item_path)?;
        let mut attachments: HashSet<String> = HashSet::new();
        let entries = fs::read_dir(&dir)
            .map_err(|e| AppError::FsError(format!("Failed to read directory: {e}")))?;
        for entry in entries {
            let entry =
                entry.map_err(|e| AppError::FsError(format!("Failed to read directory: {e}")))?;
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            attachments.insert(fname_str.to_string());
        }
        attachments.remove(METADATA_FILE_NAME);
        Ok(AttachmentsList {
            path: dir.to_string_lossy().to_string(),
            attachments,
        })
    }

    pub fn read_attachment(&self, item_path: &str, name: &str) -> Result<String, AppError> {
        let path = self.item_dir(item_path)?.join(name);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Attachment {name} does not exist"
            )));
        }
        fs::read_to_string(path)
            .map_err(|e| AppError::FsError(format!("Failed to read {name}: {e}")))
    }

    /// Save attachment file
    pub fn save_attachment_by_item_path(
        &self,
        item_path: &str,
        filename: &str,
        bytes: &[u8],
    ) -> Result<(), AppError> {
        let dir = self.item_dir(item_path)?;
        self.save_attachment(&dir, filename, bytes)
    }

    /// Save attachment file
    pub fn save_attachment(
        &self,
        dir: &PathBuf,
        filename: &str,
        bytes: &[u8],
    ) -> Result<(), AppError> {
        Self::create_dir(dir)?;
        let path = dir.join(filename);
        fs::write(&path, bytes)
            .map_err(|e| AppError::FsError(format!("Failed to write {filename}: {e}")))?;
        Ok(())
    }

    /// Copy attachment file from cursor
    pub fn copy_from_cursor(
        &self,
        cursor: Cursor<Bytes>,
        dir: &PathBuf,
        filename: &str,
    ) -> Result<(), AppError> {
        Self::create_dir(dir)?;
        let path = dir.join(filename);
        let mut file = File::create(&path)
            .map_err(|e| AppError::FsError(format!("Failed to create {filename}: {e}")))?;
        file.write_all(&cursor.into_inner())
            .map_err(|e| AppError::FsError(format!("Failed to write {filename}: {e}")))?;
        Ok(())
    }

    /// Save content.md file
    pub fn save_content(&self, content: &str, dir: &PathBuf) -> Result<(), AppError> {
        self.save_attachment(dir, CONTENT_FILE_NAME, content.as_bytes())
    }

    /// Save banner
    pub fn save_banner(&self, bytes: &[u8], dir: &PathBuf) -> Result<(), AppError> {
        self.save_attachment(dir, BANNER_FILE_NAME, bytes)
    }

    /// Delete attachment file
    pub fn delete_attachment(&self, item_path: &str, name: &str) -> Result<(), AppError> {
        let path = self.item_dir(item_path)?.join(name);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "Attachment {name} does not exist"
            )));
        }
        fs::remove_file(path)
            .map_err(|e| AppError::FsError(format!("Failed to delete {name}: {e}")))
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
            .map_err(|e| AppError::FsError(format!("Failed to create directory {:?}: {e}", dir)))
    }

    /// Get the directory path of an item from its identifier
    fn item_dir(&self, item_path: &str) -> Result<PathBuf, AppError> {
        let item_path = validate::validate_item_path(item_path)?;
        let year = item_path.chars().take(4).collect::<String>();
        Ok(self.data_dir.join(year).join(item_path))
    }
}
