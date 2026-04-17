use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_public: bool,
    pub wishlist_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProfileStats {
    pub album_count: i64,
    pub lent_count: i64,
    pub wishlist_count: i64,
    pub last_active: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSuggestion {
    pub display_name: String,
    pub country: String,
    pub city: String,
}

/// Lightweight user data for map markers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct UserLocation {
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub album_count: i64,
}

/// Safe user type without sensitive fields — used on the client side.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_public: bool,
    pub wishlist_public: bool,
    pub created_at: String,
}
