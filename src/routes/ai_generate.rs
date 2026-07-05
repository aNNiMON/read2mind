use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};

use crate::{
    AppState, db_index,
    error::AppError,
    jobs::ai_job,
    model::ai_features::AiFeature,
    model::attachment::{CONTENT_FILE_NAME, MINDMAP_FILE_NAME, SUMMARY_FILE_NAME},
    model::item::{Item, ItemKind},
};

#[derive(Debug, Deserialize)]
pub struct AiGenerateRequest {
    pub feature: AiFeature,
}

#[derive(Debug, Serialize)]
pub struct AiGenerateResponse {
    pub status: String,
    pub filename: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(req): Json<AiGenerateRequest>,
) -> Result<Json<AiGenerateResponse>, AppError> {
    let ai_config = state
        .config
        .ai
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("AI features are not enabled".to_owned()))?;

    let mut metadata = state.storage.read_metadata(&path)?;
    let filename = match metadata.kind {
        ItemKind::Article | ItemKind::Video => CONTENT_FILE_NAME,
        _ => {
            return Err(AppError::InvalidRequest(
                "Item does not support AI processing".to_owned(),
            ));
        }
    };

    let content = state.storage.read_attachment(&path, filename)?;
    let (filename, result) = match req.feature {
        AiFeature::Summary => (
            SUMMARY_FILE_NAME,
            ai_job::summarize(&state.client, ai_config, &content)
                .await
                .map_err(AppError::AiError)?,
        ),
        AiFeature::Mindmap => (
            MINDMAP_FILE_NAME,
            ai_job::mindmap(&state.client, ai_config, &content)
                .await
                .map_err(AppError::AiError)?,
        ),
    };

    let mut status = "skipped".to_owned();
    if let Some(text) = result {
        state
            .storage
            .save_attachment_by_item_path(&path, filename, text.as_bytes())?;
        metadata.updated_at = Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        state.storage.save_metadata(&metadata, &path)?;
        "success".clone_into(&mut status);
    }

    let item = Item::from_metadata(metadata, path);
    db_index::add_item(&state.db, &item)?;

    Ok(Json(AiGenerateResponse {
        status,
        filename: filename.to_owned(),
    }))
}
