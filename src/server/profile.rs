use crate::models::{LocationSuggestion, ProfileStats, SeriesWithOwnership, UserPublic};
use leptos::prelude::*;
use server_fn::ServerFnError;

#[server]
pub async fn get_public_profile(username: String) -> Result<Option<UserPublic>, ServerFnError> {
    let pool = crate::db::pool();

    let user = sqlx::query_as::<_, UserPublic>(
        "SELECT id, username, display_name, avatar_url, bio,
                location, latitude, longitude,
                is_public, wishlist_public, created_at
         FROM users WHERE username = ? AND is_public = 1",
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(user)
}

#[server]
pub async fn get_public_collection(
    username: String,
) -> Result<Vec<SeriesWithOwnership>, ServerFnError> {
    let pool = crate::db::pool();

    let items = sqlx::query_as::<_, SeriesWithOwnership>(
        "SELECT s.id, s.title, s.work_type,
                COALESCE((SELECT GROUP_CONCAT(DISTINCT au.display_name)
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 JOIN albums alb ON alb.id = aa.album_id
                 WHERE alb.series_id = s.id), '') as author,
                s.description, s.cover_url, s.year, s.number_of_albums,
                s.bubble_id, s.slug, s.is_terminated, s.created_at,
                COUNT(CASE WHEN ua.owned = 1 THEN ua.album_id END) as owned_count,
                (SELECT COUNT(*) FROM albums WHERE series_id = s.id) as total_albums,
                COUNT(CASE WHEN ua.owned = 1 AND ua.for_sale_price IS NOT NULL THEN ua.album_id END) as for_sale_count
         FROM series s
         JOIN albums a ON a.series_id = s.id
         JOIN user_albums ua ON ua.album_id = a.id AND ua.owned = 1
         JOIN users u ON ua.user_id = u.id
         WHERE u.username = ? AND u.is_public = 1
         GROUP BY s.id
         ORDER BY MAX(ua.owned_at) DESC",
    )
    .bind(&username)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}

#[server]
pub async fn get_public_profile_stats(username: String) -> Result<ProfileStats, ServerFnError> {
    let pool = crate::db::pool();

    let stats = sqlx::query_as::<_, ProfileStats>(
        "SELECT
            (SELECT COUNT(*) FROM user_albums ua
             JOIN users u ON ua.user_id = u.id
             WHERE u.username = ? AND u.is_public = 1 AND ua.owned = 1) as album_count,
            (SELECT COUNT(*) FROM album_loans al
             JOIN users u ON al.lender_id = u.id
             WHERE u.username = ? AND u.is_public = 1) as lent_count,
            (SELECT COUNT(*) FROM user_albums ua
             JOIN users u ON ua.user_id = u.id
             WHERE u.username = ? AND u.is_public = 1 AND ua.wishlisted = 1
             AND u.wishlist_public = 1) as wishlist_count,
            (SELECT MAX(ua.owned_at) FROM user_albums ua
             JOIN users u ON ua.user_id = u.id
             WHERE u.username = ? AND u.is_public = 1) as last_active",
    )
    .bind(&username)
    .bind(&username)
    .bind(&username)
    .bind(&username)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(stats)
}

#[server]
pub async fn search_location_suggestions(
    query: String,
) -> Result<Vec<LocationSuggestion>, ServerFnError> {
    crate::server::geocoding::search_locations(&query).await
}

#[server]
pub async fn get_public_wishlist(
    username: String,
) -> Result<Vec<crate::models::WishlistItem>, ServerFnError> {
    let pool = crate::db::pool();

    let items = sqlx::query_as::<_, crate::models::WishlistItem>(
        "SELECT a.id AS album_id, a.title AS album_title, a.slug AS album_slug,
                s.title AS series_title, a.tome, a.cover_url
         FROM user_albums ua
         JOIN albums a ON a.id = ua.album_id
         JOIN series s ON s.id = a.series_id
         JOIN users u ON ua.user_id = u.id
         WHERE u.username = ? AND u.is_public = 1 AND u.wishlist_public = 1
         AND ua.wishlisted = 1
         ORDER BY s.title ASC, a.tome ASC NULLS LAST",
    )
    .bind(&username)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}
