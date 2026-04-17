use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesToml {
    pub title: String,
    pub work_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_albums: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bubble_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_terminated: Option<bool>,
    #[serde(default)]
    pub albums: Vec<AlbumToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumToml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tome: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ean: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bubble_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_pages: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<AlbumAuthorToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumAuthorToml {
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Standalone author file for data/authors/*.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorToml {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bubble_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_birth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_death: Option<String>,
}
