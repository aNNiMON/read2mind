use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::{
    AppState,
    error::AppError,
    model::attachment::{CONTENT_FILE_NAME, NOTE_FILE_NAME},
    storage::AttachmentsList,
};

#[derive(Debug, Serialize)]
pub struct AttachmentInfo {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct AttachmentsResponse {
    pub path: String,
    pub content: bool,
    pub note: bool,
    pub attachments: Vec<String>,
    pub metadata: Vec<AttachmentInfo>,
}

impl From<AttachmentsList> for AttachmentsResponse {
    fn from(value: AttachmentsList) -> Self {
        let mut att = value.attachments;
        Self {
            path: value.path,
            metadata: att
                .iter()
                .map(|(name, meta)| AttachmentInfo {
                    name: name.clone(),
                    size: meta.size,
                })
                .collect(),
            content: att.remove(CONTENT_FILE_NAME).is_some(),
            note: att.remove(NOTE_FILE_NAME).is_some(),
            attachments: att.keys().cloned().collect(),
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<AttachmentsResponse>, AppError> {
    let att = state.storage.list_attachments(&path)?;
    Ok(Json(att.into()))
}
