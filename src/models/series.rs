use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Series {
    pub id: i64,
    pub title: String,
    pub work_type: String,
    pub author: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub year: Option<i32>,
    pub number_of_albums: Option<i32>,
    pub bubble_id: Option<String>,
    pub slug: String,
    pub is_terminated: Option<bool>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Album {
    pub id: i64,
    pub series_id: i64,
    pub title: Option<String>,
    pub tome: Option<i32>,
    pub cover_url: Option<String>,
    pub ean: Option<String>,
    pub bubble_id: Option<String>,
    pub slug: String,
    pub created_at: String,
}

/// Series with ownership aggregation for collection views.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SeriesWithOwnership {
    pub id: i64,
    pub title: String,
    pub work_type: String,
    pub author: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub year: Option<i32>,
    pub number_of_albums: Option<i32>,
    pub bubble_id: Option<String>,
    pub slug: String,
    pub is_terminated: Option<bool>,
    pub created_at: String,
    pub owned_count: i64,
    pub total_albums: i64,
    pub for_sale_count: i64,
}

impl From<SeriesWithOwnership> for Series {
    fn from(swo: SeriesWithOwnership) -> Self {
        Self {
            id: swo.id,
            title: swo.title,
            work_type: swo.work_type,
            author: swo.author,
            description: swo.description,
            cover_url: swo.cover_url,
            year: swo.year,
            number_of_albums: swo.number_of_albums,
            bubble_id: swo.bubble_id,
            slug: swo.slug,
            is_terminated: swo.is_terminated,
            created_at: swo.created_at,
        }
    }
}

/// Full album detail for the album page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDetail {
    pub id: i64,
    pub series_id: i64,
    pub title: Option<String>,
    pub tome: Option<i32>,
    pub cover_url: Option<String>,
    pub ean: Option<String>,
    pub slug: String,
    pub owned: bool,
    pub wishlisted: bool,
    pub for_sale_price: Option<f64>,
    // Parent series info
    pub series_title: String,
    pub series_slug: String,
    // Enriched from external API
    pub summary: Option<String>,
    pub publisher: Option<String>,
    pub number_of_pages: Option<i64>,
    pub publication_date: Option<String>,
    pub authors: Vec<AlbumAuthor>,
    pub lent_to: Option<(i64, String)>, // (user_id, display_name)
    pub borrowed_from: Option<(i64, String)>, // (user_id, display_name)
}

/// Author metadata from an album's credits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumAuthor {
    pub display_name: String,
    pub role: Option<String>,
    pub slug: String,
    pub date_birth: Option<String>,
    pub date_death: Option<String>,
}

/// Author from the local database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub display_name: String,
    pub slug: String,
    pub bio: Option<String>,
    pub date_birth: Option<String>,
    pub date_death: Option<String>,
}

/// Author profile info from an external metadata provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Bubble BD object identifier.
    pub bubble_id: String,
    pub display_name: String,
    pub image_url: Option<String>,
    pub year_of_birth: Option<String>,
    pub year_of_death: Option<String>,
    /// Full URL to the author's page on the external source, if available.
    pub external_url: Option<String>,
    /// Human-readable label for the source (e.g. "Bubble BD"), for display.
    pub source_label: Option<String>,
}

/// Album with per-user ownership flag for series detail view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AlbumWithOwnership {
    pub id: i64,
    pub series_id: i64,
    pub title: Option<String>,
    pub tome: Option<i32>,
    pub cover_url: Option<String>,
    pub ean: Option<String>,
    pub bubble_id: Option<String>,
    pub slug: String,
    pub created_at: String,
    pub owned: bool,
    pub borrowed: bool,
    pub lent: bool,
    pub wishlisted: bool,
    pub for_sale_price: Option<f64>,
}
