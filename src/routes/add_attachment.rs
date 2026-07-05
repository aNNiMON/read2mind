use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
};
use chrono::{Local, SecondsFormat};
use serde::Serialize;

use crate::{AppState, db_index, error::AppError, model::item::Item, validate};

#[derive(Debug, Serialize)]
pub struct AddAttachmentResponse {
    pub status: String,
    pub filename: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    multipart: Multipart,
) -> Result<Json<AddAttachmentResponse>, AppError> {
    let mut metadata = state.storage.read_metadata(&path)?;

    let (filename, file_bytes) = parse_multipart(multipart).await?;
    let filename = validate::validate_attachment_filename(&filename)?;

    state
        .storage
        .save_attachment_by_item_path(&path, filename, &file_bytes)?;

    // Update index
    metadata.updated_at = Some(Local::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    state.storage.save_metadata(&metadata, &path)?;
    let item = Item::from_metadata(metadata, path);
    db_index::add_item(&state.db, &item)?;

    Ok(Json(AddAttachmentResponse {
        status: "success".to_owned(),
        filename: filename.to_owned(),
    }))
}

async fn parse_multipart(mut multipart: Multipart) -> Result<(String, Bytes), AppError> {
    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InvalidRequest(format!("Invalid multipart request: {e}")))?
    {
        let field_name = field.name().ok_or_else(|| {
            AppError::InvalidRequest("Multipart field is missing a name".to_owned())
        })?;

        match field_name {
            "file" => {
                if file_bytes.is_some() {
                    return Err(AppError::InvalidRequest(
                        "Multipart request contains more than one file field".to_owned(),
                    ));
                }
                let field_filename = field.file_name().ok_or_else(|| {
                    AppError::InvalidRequest("Multipart field is missing a filename".to_owned())
                })?;
                filename = Some(field_filename.to_owned());
                file_bytes =
                    Some(field.bytes().await.map_err(|e| {
                        AppError::InvalidRequest(format!("Invalid file field: {e}"))
                    })?);
            }
            other => {
                return Err(AppError::InvalidRequest(format!(
                    "Unexpected multipart field: {other}",
                )));
            }
        }
    }
    let filename = filename
        .ok_or_else(|| AppError::InvalidRequest("Missing filename multipart field".to_owned()))?;
    let file_bytes = file_bytes
        .ok_or_else(|| AppError::InvalidRequest("Missing file multipart field".to_owned()))?;
    Ok((filename, file_bytes))
}
