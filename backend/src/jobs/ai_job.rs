use reqwest::Client;
use serde_json::json;

use crate::config::AiConfig;

pub type AiResponse = Option<String>;

/// Summarize given content
pub async fn summarize(
    client: &Client,
    config: &AiConfig,
    content: &str,
) -> Result<AiResponse, String> {
    if let (Some(prompt), Some(model)) = (&config.summarize_prompt, &config.summarize_model) {
        let prompt = prompt.replace("{content}", content);
        chat_completion(client, config, model, &prompt)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}

/// Generate an AI mindmap from the given content
pub async fn mindmap(
    client: &Client,
    config: &AiConfig,
    content: &str,
) -> Result<AiResponse, String> {
    if let (Some(prompt), Some(model)) = (&config.mindmap_prompt, &config.mindmap_model) {
        let prompt = prompt.replace("{content}", content);
        chat_completion(client, config, model, &prompt)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}

/// Call an OpenAI-compatible chat-completions endpoint
async fn chat_completion(
    client: &Client,
    config: &AiConfig,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let mut req = client.post(&url).json(&body);
    if let Ok(key) = std::env::var(&config.api_key_env)
        && !key.is_empty()
    {
        req = req.bearer_auth(key);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("read body failed: {e}"))?;
    if !status.is_success() {
        return Err(format!("AI endpoint returned {status}: {text}"));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("parse response failed: {e}"))?;
    parsed["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| format!("unexpected response: {text}"))
}
