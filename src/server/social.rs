use crate::models::{AlbumLoan, Notification, UserLocation, UserPublic};
use leptos::prelude::*;
use server_fn::ServerFnError;

/// Search public users by username or display name.
#[server]
pub async fn search_users(query: String) -> Result<Vec<UserPublic>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let current_user_id = get_current_user().await?.map(|u| u.id).unwrap_or(-1);
    let pattern = format!("%{}%", query.trim());

    let users = sqlx::query_as::<_, UserPublic>(
        "SELECT id, username, display_name, avatar_url, bio,
                location, latitude, longitude,
                is_public, wishlist_public, created_at
         FROM users
         WHERE is_public = 1 AND id != ?
           AND (username LIKE ? OR display_name LIKE ?)
         ORDER BY display_name ASC
         LIMIT 10",
    )
    .bind(current_user_id)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(users)
}

/// Follow a user (one-way). Creates a notification for the target.
#[server]
pub async fn follow_user(target_id: i64) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    if user.id == target_id {
        return Err(ServerFnError::new("Cannot follow yourself"));
    }

    let result = sqlx::query("INSERT OR IGNORE INTO follows (user_id, following_id) VALUES (?, ?)")
        .bind(user.id)
        .bind(target_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if result.rows_affected() > 0 {
        let payload = serde_json::json!({
            "from_user_id": user.id,
            "from_username": user.username,
            "from_display_name": user.display_name,
        });
        sqlx::query("INSERT INTO notifications (user_id, type, payload) VALUES (?, 'followed', ?)")
            .bind(target_id)
            .bind(payload.to_string())
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    }

    Ok(())
}

/// Unfollow a user. Also removes any album loans where current user is lender
/// and the unfollowed user is borrower.
#[server]
pub async fn unfollow_user(target_id: i64) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    sqlx::query("DELETE FROM follows WHERE user_id = ? AND following_id = ?")
        .bind(user.id)
        .bind(target_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    // Cascade: remove loans where current user lent to this person
    sqlx::query("DELETE FROM album_loans WHERE lender_id = ? AND borrower_id = ?")
        .bind(user.id)
        .bind(target_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(())
}

/// Check if the current user is following a given user.
#[server]
pub async fn is_following(user_id: i64) -> Result<bool, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let current = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(false),
    };

    let exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM follows WHERE user_id = ? AND following_id = ?",
    )
    .bind(current.id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    Ok(exists)
}

/// Get the list of users the current user is following.
#[server]
pub async fn get_following() -> Result<Vec<UserPublic>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let following = sqlx::query_as::<_, UserPublic>(
        "SELECT u.id, u.username, u.display_name, u.avatar_url, u.bio,
                u.location, u.latitude, u.longitude,
                u.is_public, u.wishlist_public, u.created_at
         FROM users u
         JOIN follows f ON f.following_id = u.id
         WHERE f.user_id = ?
         ORDER BY u.display_name ASC",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(following)
}

/// Get the number of users the current user is following.
#[server]
pub async fn get_following_count() -> Result<i64, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(0),
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM follows WHERE user_id = ?")
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(count)
}

/// Get the number of albums currently lent out by the current user.
#[server]
pub async fn get_lent_album_count() -> Result<i64, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(0),
    };

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM album_loans WHERE lender_id = ?")
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(count)
}

/// Get all albums currently lent out by the current user.
#[server]
pub async fn get_lent_albums() -> Result<Vec<AlbumLoan>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let rows = sqlx::query_as::<_, AlbumLoan>(
        "SELECT al.id, al.album_id,
                a.title AS album_title,
                a.slug AS album_slug,
                s.title AS series_title,
                a.cover_url,
                al.borrower_id,
                u.display_name AS borrower_display_name,
                u.username AS borrower_username,
                al.created_at
         FROM album_loans al
         JOIN albums a ON a.id = al.album_id
         JOIN series s ON s.id = a.series_id
         JOIN users u ON u.id = al.borrower_id
         WHERE al.lender_id = ?
         ORDER BY al.created_at DESC",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(rows)
}

/// Get unread notification count for the current user.
#[server]
pub async fn get_unread_count() -> Result<i64, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(0),
    };

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = ? AND read = 0")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(count)
}

/// Get recent notifications for the current user (last 20).
#[server]
pub async fn get_notifications() -> Result<Vec<Notification>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let items = sqlx::query_as::<_, Notification>(
        "SELECT id, type, payload, read, created_at
         FROM notifications
         WHERE user_id = ?
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}

/// Mark all notifications as read for the current user.
#[server]
pub async fn mark_notifications_read() -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    sqlx::query("UPDATE notifications SET read = 1 WHERE user_id = ? AND read = 0")
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(())
}

/// Delete all notifications for the current user.
#[server]
pub async fn clear_notifications() -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    sqlx::query("DELETE FROM notifications WHERE user_id = ?")
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(())
}

/// Lend an album to a followed user. Validates ownership and follow status.
#[server]
pub async fn lend_album(album_id: i64, borrower_id: i64) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    // Verify ownership
    let owns: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM user_albums WHERE user_id = ? AND album_id = ?",
    )
    .bind(user.id)
    .bind(album_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !owns {
        return Err(ServerFnError::new("You don't own this album"));
    }

    // Verify follow
    let following: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM follows WHERE user_id = ? AND following_id = ?",
    )
    .bind(user.id)
    .bind(borrower_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if !following {
        return Err(ServerFnError::new("Not following this user"));
    }

    // Insert loan
    sqlx::query("INSERT INTO album_loans (lender_id, borrower_id, album_id) VALUES (?, ?, ?)")
        .bind(user.id)
        .bind(borrower_id)
        .bind(album_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create loan: {e}")))?;

    // Notify borrower
    let album_title: Option<String> = sqlx::query_scalar(
        "SELECT COALESCE(a.title, s.title) FROM albums a JOIN series s ON s.id = a.series_id WHERE a.id = ?",
    )
    .bind(album_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let album_slug: Option<String> = sqlx::query_scalar("SELECT slug FROM albums WHERE id = ?")
        .bind(album_id)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();

    let payload = serde_json::json!({
        "lender_display_name": user.display_name,
        "album_title": album_title,
        "album_id": album_id,
        "album_slug": album_slug,
    });
    sqlx::query("INSERT INTO notifications (user_id, type, payload) VALUES (?, 'album_lent', ?)")
        .bind(borrower_id)
        .bind(payload.to_string())
        .execute(&pool)
        .await
        .ok();

    Ok(())
}

/// Return a loaned album. Only the lender can do this.
#[server]
pub async fn return_album(loan_id: i64) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let deleted = sqlx::query("DELETE FROM album_loans WHERE id = ? AND lender_id = ?")
        .bind(loan_id)
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if deleted.rows_affected() == 0 {
        return Err(ServerFnError::new("Loan not found or not authorized"));
    }

    Ok(())
}

/// Get loan ID for an album where the current user is the lender.
#[server]
pub async fn get_loan_id(album_id: i64) -> Result<Option<i64>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM album_loans WHERE album_id = ? AND lender_id = ?",
    )
    .bind(album_id)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(id)
}

/// Get all public users with location data for the world map.
#[server]
pub async fn get_public_user_locations() -> Result<Vec<UserLocation>, ServerFnError> {
    let pool = crate::db::pool();

    let users = sqlx::query_as::<_, UserLocation>(
        "SELECT u.username, u.display_name, u.avatar_url,
                u.latitude, u.longitude,
                COUNT(ua.album_id) AS album_count
         FROM users u
         LEFT JOIN user_albums ua ON ua.user_id = u.id
         WHERE u.is_public = 1
           AND u.latitude IS NOT NULL
           AND u.longitude IS NOT NULL
         GROUP BY u.id
         ORDER BY u.display_name ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(users)
}
