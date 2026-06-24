use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// Article with url and content
    #[default]
    Article,
    /// Bookmark with url only
    Bookmark,
    /// Note with markdown content
    Note,
    /// Task with markdown content
    Task,
    /// Video with youtube url and transcription
    Video,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    /// Default for all items
    #[default]
    New,
    /// Planned to read, watch or to do
    Planned,
    /// In progress or reading, watching
    InProgress,
    Paused,
    /// Item is done
    Done,
    /// Item is not relevant anymore
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: ItemKind,
    pub path: String,
    pub title: String,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub status: ItemStatus,
    pub created_at: String,
    pub updated_at: Option<String>,
}
