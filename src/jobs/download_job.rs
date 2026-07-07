use std::io::Cursor;

use axum::body::Bytes;
use reqwest::Client;

pub struct DownloadJob {
    pub client: Client,
    pub url: String,
}

impl DownloadJob {
    pub fn new(client: Client, url: String) -> Self {
        Self { client, url }
    }

    pub async fn run(&self) -> Result<Cursor<Bytes>, String> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| format!("download {}: {}", self.url, e))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("bytes {}: {}", self.url, e))?;

        Ok(Cursor::new(bytes))
    }
}
