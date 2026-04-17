//! # bubblebd
//!
//! A Rust client for the [Bubble BD](https://www.bubblebd.com) API — a French
//! platform for comics, manga, and bande dessinée metadata.
//!
//! ## API layers
//!
//! Bubble BD exposes two complementary layers:
//!
//! | Layer | Base URL | Purpose |
//! |-------|----------|---------|
//! | **Algolia search** | `https://{app_id}-dsn.algolia.net` | Fast full-text search across 6 indexes |
//! | **REST API** | `https://api.bubblebd.com/v1.6` | Detailed resource lookups by object ID |
//!
//! ### Algolia search indexes
//!
//! | Index | Method | Hit type |
//! |-------|--------|----------|
//! | `Series` | [`Client::search_series`] | [`SeriesHit`] |
//! | `Albums` | [`Client::search_albums_by_ean`] | [`AlbumHit`] |
//! | `Authors` | [`Client::search_authors`] | [`AuthorHit`] |
//! | `Publishers` | [`Client::search_publishers`] | [`PublisherHit`] |
//! | `Tags` | [`Client::search_tags`] | [`TagHit`] |
//! | `Collections` | [`Client::search_collections`] | [`CollectionHit`] |
//!
//! ### REST endpoints
//!
//! | Endpoint | Method | Return type |
//! |----------|--------|-------------|
//! | `GET /v1.6/series/{id}` | [`Client::get_series`] | [`Series`] |
//! | `GET /v1.6/albums/{id}` | [`Client::get_album`] | [`Album`] |
//!
//! ## Quick start
//!
#![doc = concat!("```no_run\n", include_str!("../examples/quickstart.rs"), "\n```")]
//!
//! ## Custom base URLs (testing)
//!
//! Use [`Client::with_base_urls`] to point at a mock server (e.g.
//! [`mockito`](https://docs.rs/mockito)):
//!
//! ```
//! let client = bubblebd::Client::with_base_urls(
//!     "http://localhost:1234",  // Algolia mock
//!     "http://localhost:5678",  // REST API mock
//! );
//! ```

mod album;
mod error;
mod search;
mod series;
mod types;

pub use error::Error;
pub use types::*;

/// Convenience alias for `Result<T, bubblebd::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_ALGOLIA_APP_ID: &str = "6A891Z72ZD";
const DEFAULT_ALGOLIA_API_KEY: &str = "7b4e51bd140b7d73c3241e3a00984f5b";
const DEFAULT_API_BASE_URL: &str = "https://api.bubblebd.com";

/// Client for the Bubble BD API (Algolia search + REST details).
///
/// Create one instance and reuse it — it holds a [`reqwest::Client`] connection
/// pool internally.
///
/// # Examples
///
/// ```
/// // Production client (default URLs)
/// let client = bubblebd::Client::new();
///
/// // Test client pointing at a mock server
/// let test_client = bubblebd::Client::with_base_urls(
///     "http://localhost:1234",
///     "http://localhost:5678",
/// );
/// ```
pub struct Client {
    http: reqwest::Client,
    algolia_app_id: String,
    algolia_api_key: String,
    algolia_base_url: String,
    api_base_url: String,
}

impl Client {
    /// Create a client with default production URLs.
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            algolia_app_id: DEFAULT_ALGOLIA_APP_ID.to_string(),
            algolia_api_key: DEFAULT_ALGOLIA_API_KEY.to_string(),
            algolia_base_url: format!("https://{DEFAULT_ALGOLIA_APP_ID}-dsn.algolia.net"),
            api_base_url: DEFAULT_API_BASE_URL.to_string(),
        }
    }

    /// Create a client pointing at custom base URLs (for testing with mockito).
    pub fn with_base_urls(algolia_base_url: &str, api_base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            algolia_app_id: DEFAULT_ALGOLIA_APP_ID.to_string(),
            algolia_api_key: DEFAULT_ALGOLIA_API_KEY.to_string(),
            algolia_base_url: algolia_base_url.to_string(),
            api_base_url: api_base_url.to_string(),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
