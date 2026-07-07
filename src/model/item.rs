use core::fmt;

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
    /// Item is paused
    Paused,
    /// Item is done
    Done,
    /// Item is not relevant anymore
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ItemMetadata {
    pub kind: ItemKind,
    pub title: String,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub status: ItemStatus,
    pub created_at: String,
    pub updated_at: Option<String>,
    /// Could be in any format, indexing this is disconnected
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Item {
    pub kind: ItemKind,
    pub path: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub status: ItemStatus,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<Item> for ItemMetadata {
    fn from(item: Item) -> Self {
        Self {
            kind: item.kind,
            title: item.title,
            url: item.url,
            tags: item.tags,
            author: item.author,
            status: item.status,
            created_at: item.created_at,
            updated_at: item.updated_at,
            published_at: None,
        }
    }
}

impl Item {
    pub fn from_metadata(m: ItemMetadata, path: String) -> Self {
        Self {
            kind: m.kind,
            path,
            title: m.title,
            url: m.url,
            tags: m.tags,
            author: m.author,
            status: m.status,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Article => "article",
                Self::Bookmark => "bookmark",
                Self::Note => "note",
                Self::Task => "task",
                Self::Video => "video",
            }
        )
    }
}

impl From<&str> for ItemKind {
    fn from(s: &str) -> Self {
        match s {
            "article" => Self::Article,
            "bookmark" => Self::Bookmark,
            "note" => Self::Note,
            "task" => Self::Task,
            "video" => Self::Video,
            _ => panic!("unknown item kind: {}", s),
        }
    }
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::New => "new",
                Self::Planned => "planned",
                Self::InProgress => "in_progress",
                Self::Paused => "paused",
                Self::Done => "done",
                Self::Rejected => "rejected",
            }
        )
    }
}

impl From<&str> for ItemStatus {
    fn from(s: &str) -> Self {
        match s {
            "new" => Self::New,
            "planned" => Self::Planned,
            "in_progress" => Self::InProgress,
            "paused" => Self::Paused,
            "done" => Self::Done,
            "rejected" => Self::Rejected,
            _ => panic!("unknown item status: {}", s),
        }
    }
}
