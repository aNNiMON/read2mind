use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind_address: String,
    #[serde(default)]
    pub api_secret: Option<String>,
    #[serde(default)]
    pub ai: Option<AiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub base_url: String,
    pub api_key_env: String,
    #[serde(default)]
    pub summarize_prompt: Option<String>,
    #[serde(default)]
    pub summarize_model: Option<String>,
    #[serde(default)]
    pub mindmap_prompt: Option<String>,
    #[serde(default)]
    pub mindmap_model: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8555".to_owned(),
            api_secret: None,
            ai: Some(Default::default()),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_owned(),
            api_key_env: "OPENAI_API_KEY".to_owned(),
            summarize_prompt: Default::default(),
            summarize_model: Some("gpt-5-nano".to_owned()),
            mindmap_prompt: Default::default(),
            mindmap_model: Some("gpt-5-nano".to_owned()),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let mut config: Config = toml::from_str(&content)?;
            // Hash api_secret
            if let Some(api_secret) = &config.api_secret {
                let digest = Sha256::digest(api_secret.as_bytes());
                let hashed: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
                config.api_secret = Some(hashed);
            }
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
