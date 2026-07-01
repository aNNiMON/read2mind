use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::{AppState, attachment::CONTENT_FILE_NAME, error::AppError, storage::AttachmentsList};

#[derive(Debug, Serialize)]
pub struct AttachmentsResponse {
    pub path: String,
    pub content: bool,
    pub attachments: Vec<String>,
}

impl From<AttachmentsList> for AttachmentsResponse {
    fn from(value: AttachmentsList) -> Self {
        let mut att = value.attachments;
        Self {
            path: value.path,
            content: att.remove(CONTENT_FILE_NAME),
            attachments: att.into_iter().collect(),
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
