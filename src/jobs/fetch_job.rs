use decruft::{DecruftOptions, DecruftResult};

pub struct FetchJob {
    pub url: String,
    pub html: Option<String>,
}

impl FetchJob {
    pub fn new(url: String, html: Option<String>) -> Self {
        Self { url, html }
    }

    pub fn from_url(url: String) -> Self {
        Self { url, html: None }
    }

    pub fn run(&self) -> Result<DecruftResult, String> {
        let html = match &self.html {
            Some(html) => html.clone(),
            None => decruft::fetch_page(&self.url).map_err(|e| format!("fetch_page: {e}"))?,
        };

        let mut options = DecruftOptions::default();
        options.url = Some(self.url.clone());
        options.include_replies = false;
        options.separate_markdown = false;
        options.markdown = true;
        Ok(decruft::parse(&html, &options))
    }
}
