use serde::{Deserialize, Serialize};

/// A notification for the current user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Notification {
    pub id: i64,
    #[cfg_attr(feature = "ssr", sqlx(rename = "type"))]
    pub notification_type: String,
    pub payload: String,
    pub read: bool,
    pub created_at: String,
}

/// An active album loan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AlbumLoan {
    pub id: i64,
    pub album_id: i64,
    pub album_title: Option<String>,
    pub album_slug: String,
    pub series_title: String,
    pub cover_url: Option<String>,
    pub borrower_id: i64,
    pub borrower_display_name: String,
    pub borrower_username: String,
    pub created_at: String,
}
