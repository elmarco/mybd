#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API returned status {0}")]
    Status(u16),

    #[error("Failed to parse XML: {0}")]
    Xml(String),
}
