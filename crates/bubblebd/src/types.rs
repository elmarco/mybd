use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared enums
// ---------------------------------------------------------------------------

/// The type/category of a work in the Bubble BD catalog.
///
/// Bubble BD classifies series into broad categories. This enum normalizes the
/// raw `category` and `type` strings returned by the API into three variants.
///
/// # Mapping rules
///
/// | API value (case-insensitive) | Variant |
/// |-----------------------------|---------|
/// | `"mangas"`, `"manga"` | [`Manga`](WorkType::Manga) |
/// | `"comics"` | [`Comic`](WorkType::Comic) |
/// | everything else (`"bd"`, `"jeunesse"`, …) | [`Bd`](WorkType::Bd) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkType {
    /// Franco-Belgian bande dessinée (default).
    Bd,
    /// Japanese manga.
    Manga,
    /// American or other comics.
    Comic,
}

impl WorkType {
    /// Map a Bubble BD `category` + `type` field pair to a [`WorkType`].
    ///
    /// The `category` field takes precedence. If it is `None`, the `type`
    /// field is checked as a fallback. Unrecognized values default to
    /// [`WorkType::Bd`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bubblebd::WorkType;
    ///
    /// assert_eq!(WorkType::from_category(Some("mangas"), None), WorkType::Manga);
    /// assert_eq!(WorkType::from_category(Some("comics"), None), WorkType::Comic);
    /// assert_eq!(WorkType::from_category(Some("bd"), None), WorkType::Bd);
    /// assert_eq!(WorkType::from_category(None, Some("manga edition")), WorkType::Manga);
    /// assert_eq!(WorkType::from_category(None, None), WorkType::Bd);
    /// ```
    pub fn from_category(category: Option<&str>, type_field: Option<&str>) -> Self {
        match category {
            Some(c) if c.eq_ignore_ascii_case("mangas") || c.eq_ignore_ascii_case("manga") => {
                WorkType::Manga
            }
            Some(c) if c.eq_ignore_ascii_case("comics") => WorkType::Comic,
            Some(_) => WorkType::Bd,
            None => match type_field {
                Some(t) if t.to_lowercase().contains("manga") => WorkType::Manga,
                Some(t) if t.to_lowercase().contains("comic") => WorkType::Comic,
                _ => WorkType::Bd,
            },
        }
    }
}

impl std::fmt::Display for WorkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkType::Bd => write!(f, "bd"),
            WorkType::Manga => write!(f, "manga"),
            WorkType::Comic => write!(f, "comic"),
        }
    }
}

// ---------------------------------------------------------------------------
// Shared value types (used across Algolia hits and REST responses)
// ---------------------------------------------------------------------------

/// A genre/theme tag attached to a series or album.
///
/// Tags are returned both from the Algolia `Tags` index (as [`TagHit`]) and
/// embedded inside [`Series`] and [`Album`] REST responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// Human-readable tag name (e.g. `"Action"`, `"Humour"`).
    pub name: String,
}

/// A publisher (e.g. Glénat, Dargaud).
///
/// Appears embedded in [`Print`] objects within album and series responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// Publisher display name.
    pub name: String,
}

/// An author or artist with biographical details.
///
/// Embedded in [`Print`] objects within album/series REST responses.
/// For lighter search results, see [`AuthorHit`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// URL-friendly slug (e.g. `"yuto-suzuki"`).
    pub permalink: Option<String>,
    /// Preferred display name.
    pub display_name: String,
    /// Author's role (e.g. `"scénariste"`, `"dessinateur"`, `"auteur"`).
    pub role: Option<String>,
    /// First name, when known.
    pub first_name: Option<String>,
    /// Last name, when known.
    pub last_name: Option<String>,
    /// Profile image URL.
    pub image_url: Option<String>,
    /// Birth year as a string (e.g. `"1907"`).
    pub year_of_birth: Option<String>,
    /// Death year as a string, if deceased.
    pub year_of_death: Option<String>,
    /// Short biography.
    pub biography: Option<String>,
}

/// Stock availability for a [`Print`] edition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Availability {
    /// Human-readable status (e.g. `"En stock"`, `"72 librairies"`).
    pub message: String,
    /// Availability code (`100` = available).
    pub code: i64,
    /// Number of sellers offering this print.
    pub number_of_sellers: i64,
}

/// Pricing and availability for a [`Print`] edition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellingInfo {
    /// Cover price as a string (e.g. `"7.20"`).
    pub price: Option<String>,
    /// Discounted price, if a promotion is active.
    pub discounted_price: Option<String>,
    /// Online ordering availability.
    pub online: Option<Availability>,
    /// Click-and-collect (in-store pickup) availability.
    pub click_and_collect: Option<Availability>,
}

/// A physical print edition of an album.
///
/// An album can have multiple prints (hardcover, paperback, special edition).
/// Each print carries its own EAN/ISBN, physical dimensions, publisher,
/// author list, and availability info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Print {
    /// Bubble BD object identifier for this print.
    pub object_id: String,
    /// EAN-13 barcode (e.g. `"9782344067321"`).
    pub ean: Option<String>,
    /// ISBN, when distinct from the EAN.
    pub isbn: Option<String>,
    /// Publication date as an ISO 8601 string.
    pub publication_date: Option<String>,
    /// Number of pages.
    pub number_of_pages: Option<i64>,
    /// Length in cm.
    pub length_cm: Option<f64>,
    /// Height in cm.
    pub height_cm: Option<f64>,
    /// Width in cm.
    pub width_cm: Option<f64>,
    /// Weight in kg.
    pub weight_kg: Option<f64>,
    /// Publisher of this edition.
    pub publisher: Option<Publisher>,
    /// Edition type (e.g. `"album simple N&B"`).
    pub print_type: Option<String>,
    /// Publisher collection/imprint (e.g. `"Glénat Shonen Manga"`).
    pub collection: Option<String>,
    /// Cover image URL (largest available front image).
    pub cover_url: Option<String>,
    /// Authors credited on this print.
    pub authors: Vec<Author>,
    /// Pricing and stock availability.
    pub selling_info: Option<SellingInfo>,
}

/// Back-reference to the parent series from an [`Album`] response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSerie {
    /// Series object identifier.
    pub object_id: String,
    /// Series title.
    pub title: String,
    /// Average user rating (0–5 scale).
    pub note: Option<f64>,
    /// Number of user ratings.
    pub number_of_notes: Option<i64>,
    /// Raw category string (e.g. `"Mangas"`, `"bd"`).
    pub category: Option<String>,
    /// URL-friendly slug.
    pub permalink: Option<String>,
}

// ---------------------------------------------------------------------------
// Algolia search hit types
// ---------------------------------------------------------------------------

/// A search hit from the Algolia **Series** index.
///
/// Returned by [`Client::search_series`](crate::Client::search_series).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesHit {
    /// Series object identifier (use with
    /// [`Client::get_series`](crate::Client::get_series)).
    pub object_id: String,
    /// Series title.
    pub title: String,
    /// Cover thumbnail URL.
    pub cover_url: Option<String>,
    /// Average user rating (0–5 scale).
    pub note: Option<f64>,
    /// URL-friendly slug (e.g. `"one-piece-classique-glenat"`).
    pub permalink: Option<String>,
    /// Publisher collection/imprint (e.g. `"Classique Glénat"`).
    pub collection: Option<String>,
    /// Whether the series has ended.
    pub is_terminated: Option<bool>,
    /// Edition/format type (e.g. `"album simple"`, `"hors série"`).
    pub series_type: Option<String>,
    /// Whether the series contains sexual content.
    pub has_sexual_content: Option<bool>,
}

/// A search hit from the Algolia **Albums** index (EAN barcode search).
///
/// Returned by [`Client::search_albums_by_ean`](crate::Client::search_albums_by_ean).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumHit {
    /// Album object identifier.
    pub object_id: String,
    /// Parent series object identifier.
    pub series_object_id: String,
    /// Display title (formatted as `"Series - T{n}"` when tome number is known).
    pub title: String,
    /// Cover thumbnail URL.
    pub cover_url: Option<String>,
    /// Average user rating (0–5 scale).
    pub note: Option<f64>,
    /// Number of user ratings.
    pub number_of_notes: Option<i64>,
    /// First EAN barcode from the semicolon-separated `eans` field.
    pub ean: Option<String>,
    /// Volume/tome number within the series.
    pub tome: Option<i64>,
    /// URL-friendly slug (e.g. `"tintin-tome-2-tintin-au-congo"`).
    pub permalink: Option<String>,
    /// Cover price as a string (e.g. `"12.50"`).
    pub price: Option<String>,
    /// Raw series title (e.g. `"Tintin"`).
    pub serie_title: Option<String>,
    /// URL-friendly slug of the parent series.
    pub serie_permalink: Option<String>,
    /// Object identifier of the default selling print edition.
    pub default_selling_print_object_id: Option<String>,
    /// Whether the album contains sexual content.
    pub has_sexual_content: Option<bool>,
}

/// A search hit from the Algolia **Authors** index.
///
/// Returned by [`Client::search_authors`](crate::Client::search_authors).
///
/// For full biographical details, fetch an album or series that includes this
/// author — the author data is embedded in [`Print`] objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorHit {
    /// Author object identifier.
    pub object_id: String,
    /// URL-friendly slug (e.g. `"herge"`).
    pub permalink: Option<String>,
    /// Preferred display name.
    pub display_name: String,
    /// Profile image URL.
    pub image_url: Option<String>,
    /// Birth year as a string (e.g. `"1907"`).
    pub year_of_birth: Option<String>,
    /// Death year as a string, if deceased.
    pub year_of_death: Option<String>,
}

/// A search hit from the Algolia **Publishers** index.
///
/// Returned by [`Client::search_publishers`](crate::Client::search_publishers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherHit {
    /// Publisher object identifier.
    pub object_id: String,
    /// Publisher display name.
    pub name: String,
}

/// A search hit from the Algolia **Tags** index.
///
/// Returned by [`Client::search_tags`](crate::Client::search_tags).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagHit {
    /// Tag object identifier.
    pub object_id: String,
    /// Tag name (e.g. `"Action"`, `"Humour"`).
    pub name: String,
    /// Popularity weight — higher means more commonly used.
    pub weight: i64,
}

/// A search hit from the Algolia **Collections** index.
///
/// Collections are publisher imprint lines (e.g. "Panini Manga",
/// "Soleil Manga Seinen").
///
/// Returned by [`Client::search_collections`](crate::Client::search_collections).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionHit {
    /// Collection object identifier.
    pub object_id: String,
    /// Collection name in French.
    pub name: String,
}

// ---------------------------------------------------------------------------
// Lightweight album summary (extracted from series REST response)
// ---------------------------------------------------------------------------

/// A lightweight summary of an album within a series.
///
/// Extracted from the `albums` array in a `GET /v1.6/series/{id}` response.
/// Contains only the fields needed to populate a local album catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// Volume/tome number within the series.
    pub tome: Option<i64>,
    /// Album title (often `None` for numbered tomes).
    pub title: Option<String>,
    /// Cover image URL.
    pub cover_url: Option<String>,
    /// EAN-13 barcode (from the first print edition).
    pub ean: Option<String>,
}

// ---------------------------------------------------------------------------
// REST API detail types
// ---------------------------------------------------------------------------

/// Full album details from `GET /v1.6/albums/{object_id}`.
///
/// An album represents a single physical volume (e.g. *Sakamoto Days* Tome 21).
/// It contains one or more [`Print`] editions and a back-reference to its
/// parent [`AlbumSerie`].
///
/// Returned by [`Client::get_album`](crate::Client::get_album).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// URL-friendly slug (e.g. `"sakamoto-days-tome-21"`).
    pub permalink: Option<String>,
    /// Album title (often `None` for numbered tomes — use `tome` instead).
    pub title: Option<String>,
    /// Volume/tome number within the series.
    pub tome: Option<i64>,
    /// Synopsis or summary text.
    pub summary: Option<String>,
    /// Average user rating (0–5 scale).
    pub note: Option<f64>,
    /// Number of user ratings.
    pub number_of_notes: Option<i64>,
    /// Cover image URL (largest available front image).
    pub cover_url: Option<String>,
    /// Genre/theme tags.
    pub tags: Vec<Tag>,
    /// Available print editions.
    pub prints: Vec<Print>,
    /// Back-reference to the parent series.
    pub serie: Option<AlbumSerie>,
}

/// Full series details from `GET /v1.6/series/{object_id}`.
///
/// A series groups related [`Album`]s together (e.g. all *Tintin* volumes).
/// This type captures the most important fields from the rich API response.
///
/// Returned by [`Client::get_series`](crate::Client::get_series).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// Bubble BD object identifier.
    pub object_id: String,
    /// Series title.
    pub title: String,
    /// Normalized work type.
    pub work_type: WorkType,
    /// Short description or synopsis.
    pub description: Option<String>,
    /// EAN/ISBN of the first print of the first album.
    pub isbn: Option<String>,
    /// Cover image URL (from the first album's first print).
    pub cover_url: Option<String>,
    /// Publication year of the first album.
    pub year: Option<i32>,
    /// URL-friendly slug (e.g. `"sakamoto-days-glenat-shonen-manga"`).
    pub permalink: Option<String>,
    /// Publisher collection/imprint (e.g. `"Glénat Shonen Manga"`).
    pub collection: Option<String>,
    /// Genre string from the API (often `None`).
    pub genre: Option<String>,
    /// Whether the series has ended.
    pub is_terminated: Option<bool>,
    /// Average user rating (0–5 scale).
    pub note: Option<f64>,
    /// Number of user ratings.
    pub number_of_notes: Option<i64>,
    /// Total number of albums in the series.
    pub number_of_albums: Option<i64>,
    /// Genre/theme tags.
    pub tags: Vec<Tag>,
}
