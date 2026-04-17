mod error;
mod parse;
mod search;
mod types;

pub use error::Error;
pub use parse::parse_records;
pub use types::{Author, Record, SearchResults};

pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_BASE_URL: &str = "https://catalogue.bnf.fr";

pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
