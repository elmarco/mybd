use serde::{Deserialize, Serialize};

/// An album on the user's wishlist, with enough context for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WishlistItem {
    pub album_id: i64,
    pub album_title: Option<String>,
    pub album_slug: String,
    pub series_title: String,
    pub tome: Option<i32>,
    pub cover_url: Option<String>,
}
