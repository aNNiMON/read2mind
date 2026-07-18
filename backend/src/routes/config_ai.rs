use axum::{Json, extract::State};
use serde::Serialize;

use crate::{AppState, error::AppError, model::ai_features::AiFeature};

#[derive(Debug, Serialize)]
pub struct ConfigAIResponse {
    pub enabled: bool,
    pub features: Vec<AiFeature>,
}

pub async fn handler(State(state): State<AppState>) -> Result<Json<ConfigAIResponse>, AppError> {
    let mut enabled = false;
    let mut features: Vec<AiFeature> = Vec::new();

    if let Some(ai_config) = state.config.ai.as_ref() {
        enabled = true;
        if ai_config.summarize_prompt.is_some() && ai_config.summarize_model.is_some() {
            features.push(AiFeature::Summary);
        }
        if ai_config.mindmap_prompt.is_some() && ai_config.mindmap_model.is_some() {
            features.push(AiFeature::Mindmap);
        }
    }

    Ok(Json(ConfigAIResponse { enabled, features }))
}
