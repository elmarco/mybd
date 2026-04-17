//! Metadata provider abstraction.
//!
//! Defines a trait that external metadata backends implement, and a dispatch
//! function that routes by source string to the concrete provider.

use crate::models::{AlbumAuthor, AuthorInfo, Series};
use leptos::server_fn::ServerFnError;

// ---------------------------------------------------------------------------
// Intermediate types (provider → app domain)
// ---------------------------------------------------------------------------

/// Series metadata returned by a provider for DB insertion.
pub struct SeriesMetadata {
    pub title: String,
    pub work_type: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub year: Option<i32>,
    pub number_of_albums: Option<i32>,
}

/// Album metadata returned by a provider for DB insertion.
pub struct AlbumMetadata {
    pub provider_id: String,
    pub tome: Option<i32>,
    pub title: Option<String>,
    pub cover_url: Option<String>,
    pub ean: Option<String>,
}

/// Enrichment data for a single album (summary, authors, etc.).
pub struct AlbumEnrichment {
    pub summary: Option<String>,
    pub cover_url: Option<String>,
    pub publisher: Option<String>,
    pub number_of_pages: Option<i64>,
    pub publication_date: Option<String>,
    pub authors: Vec<AlbumAuthor>,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A metadata backend that can search and fetch series/album/author data.
pub trait MetadataProvider: Send + Sync {
    /// The source identifier (e.g. `"bubblebd"`).
    fn source_name(&self) -> &str;

    /// Human-readable label for the UI (e.g. `"Bubble BD"`).
    fn source_label(&self) -> &str;

    /// Search for series by title (first page only).
    fn search_series(
        &self,
        query: &str,
    ) -> impl Future<Output = Result<Vec<Series>, ServerFnError>> + Send;

    /// Search for series by title, fetching all pages.
    fn search_series_all(
        &self,
        query: &str,
    ) -> impl Future<Output = Result<Vec<Series>, ServerFnError>> + Send;

    /// Search for series by EAN barcode.
    fn search_by_ean(
        &self,
        ean: &str,
    ) -> impl Future<Output = Result<Vec<Series>, ServerFnError>> + Send;

    /// Search for authors by name.
    fn search_authors(
        &self,
        query: &str,
    ) -> impl Future<Output = Result<Vec<AuthorInfo>, ServerFnError>> + Send;

    /// Get detailed author info by provider-specific ID.
    fn get_author_info(
        &self,
        provider_id: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<AuthorInfo>, ServerFnError>> + Send;

    /// Fetch full series + album list for DB insertion.
    fn fetch_series_detail(
        &self,
        provider_id: &str,
    ) -> impl Future<Output = Result<(SeriesMetadata, Vec<AlbumMetadata>), ServerFnError>> + Send;

    /// Fetch enrichment data for a single album.
    fn fetch_album_enrichment(
        &self,
        provider_id: &str,
    ) -> impl Future<Output = Result<Option<AlbumEnrichment>, ServerFnError>> + Send;
}

// ---------------------------------------------------------------------------
// Bubble BD implementation
// ---------------------------------------------------------------------------

pub struct BubbleBdProvider;

impl BubbleBdProvider {
    fn hits_to_series(hits: Vec<bubblebd::SeriesHit>) -> Vec<Series> {
        hits.into_iter()
            .map(|hit| Series {
                id: 0,
                title: hit.title,
                work_type: "bd".to_string(),
                author: String::new(),
                description: hit.note.map(|n| format!("★ {n:.1}/5")),
                cover_url: hit.cover_url,
                year: None,
                number_of_albums: None,
                bubble_id: Some(hit.object_id.clone()),
                slug: String::new(),
                is_terminated: hit.is_terminated,
                created_at: String::new(),
            })
            .collect()
    }

    fn author_hit_to_info(h: bubblebd::AuthorHit) -> AuthorInfo {
        let external_url = h.permalink.as_ref().map(|p| {
            format!(
                "https://www.bubblebd.com/{p}/author/{}",
                urlencoding::encode(&h.object_id)
            )
        });
        AuthorInfo {
            bubble_id: h.object_id,
            display_name: h.display_name,
            image_url: h.image_url,
            year_of_birth: h.year_of_birth,
            year_of_death: h.year_of_death,
            external_url,
            source_label: Some("Bubble BD".to_string()),
        }
    }
}

impl MetadataProvider for BubbleBdProvider {
    fn source_name(&self) -> &str {
        "bubblebd"
    }

    fn source_label(&self) -> &str {
        "Bubble BD"
    }

    async fn search_series(&self, query: &str) -> Result<Vec<Series>, ServerFnError> {
        let client = bubblebd::Client::new();
        let hits = client
            .search_series(query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(Self::hits_to_series(hits))
    }

    async fn search_series_all(&self, query: &str) -> Result<Vec<Series>, ServerFnError> {
        let client = bubblebd::Client::new();
        let hits = client
            .search_series_all(query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(Self::hits_to_series(hits))
    }

    async fn search_by_ean(&self, ean: &str) -> Result<Vec<Series>, ServerFnError> {
        let client = bubblebd::Client::new();
        let hits = client
            .search_albums_by_ean(ean)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(hits
            .into_iter()
            .map(|hit| Series {
                id: 0,
                title: hit.serie_title.unwrap_or(hit.title),
                work_type: "bd".to_string(),
                author: String::new(),
                description: hit.note.map(|n| format!("★ {n:.1}/5")),
                cover_url: hit.cover_url,
                year: None,
                number_of_albums: None,
                bubble_id: Some(hit.series_object_id),
                slug: String::new(),
                is_terminated: None,
                created_at: String::new(),
            })
            .collect())
    }

    async fn search_authors(&self, query: &str) -> Result<Vec<AuthorInfo>, ServerFnError> {
        let client = bubblebd::Client::new();
        let hits = client
            .search_authors(query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(hits
            .into_iter()
            .take(5)
            .map(Self::author_hit_to_info)
            .collect())
    }

    async fn get_author_info(
        &self,
        provider_id: &str,
        name: &str,
    ) -> Result<Option<AuthorInfo>, ServerFnError> {
        let client = bubblebd::Client::new();
        let hits = client
            .search_authors(name)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(hits
            .into_iter()
            .find(|h| h.object_id == provider_id)
            .map(Self::author_hit_to_info))
    }

    async fn fetch_series_detail(
        &self,
        provider_id: &str,
    ) -> Result<(SeriesMetadata, Vec<AlbumMetadata>), ServerFnError> {
        let (api_series, api_albums) = bubblebd::Client::new()
            .get_series(provider_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let metadata = SeriesMetadata {
            title: api_series.title,
            work_type: api_series.work_type.to_string(),
            description: api_series.description,
            cover_url: api_series.cover_url,
            year: api_series.year,
            number_of_albums: api_series.number_of_albums.map(|n| n as i32),
        };

        let albums: Vec<AlbumMetadata> = api_albums
            .into_iter()
            .map(|a| AlbumMetadata {
                provider_id: a.object_id,
                tome: a.tome.map(|t| t as i32),
                title: a.title,
                cover_url: a.cover_url,
                ean: a.ean,
            })
            .collect();

        Ok((metadata, albums))
    }

    async fn fetch_album_enrichment(
        &self,
        provider_id: &str,
    ) -> Result<Option<AlbumEnrichment>, ServerFnError> {
        let api_album = match bubblebd::Client::new().get_album(provider_id).await {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };

        let mut enrichment = AlbumEnrichment {
            summary: api_album.summary,
            cover_url: api_album.cover_url,
            publisher: None,
            number_of_pages: None,
            publication_date: None,
            authors: vec![],
        };

        if let Some(print) = api_album.prints.first() {
            enrichment.publisher = print.publisher.as_ref().map(|p| p.name.clone());
            enrichment.number_of_pages = print.number_of_pages;
            enrichment.publication_date = print.publication_date.clone();
            enrichment.authors = print
                .authors
                .iter()
                .map(|a| AlbumAuthor {
                    display_name: a.display_name.clone(),
                    role: a.role.clone(),
                    slug: crate::server::slug::slugify(&a.display_name),
                    date_birth: a.year_of_birth.clone(),
                    date_death: a.year_of_death.clone(),
                })
                .collect();
        }

        Ok(Some(enrichment))
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

use std::future::Future;

/// Return the default metadata provider used for discovery searches.
pub fn default_provider() -> BubbleBdProvider {
    BubbleBdProvider
}
