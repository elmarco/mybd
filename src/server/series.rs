use crate::models::{AlbumAuthor, AlbumDetail, AlbumWithOwnership, Series, SeriesWithOwnership};
use leptos::prelude::*;
use server_fn::ServerFnError;

/// Resolve a Bubble BD series: look up by bubble_id in DB, or fetch from
/// the metadata provider, persist series + all albums, and return the DB series.
#[server]
pub async fn get_or_create_series(bubble_id: String) -> Result<Series, ServerFnError> {
    use crate::server::provider::{MetadataProvider, default_provider};

    let start = std::time::Instant::now();
    tracing::info!(%bubble_id, "get_or_create_series: start");
    let pool = crate::db::pool();

    // Check if already in DB
    if let Some(existing) = sqlx::query_as::<_, Series>(
        "SELECT s.id, s.title, s.work_type,
                COALESCE((SELECT GROUP_CONCAT(DISTINCT au.display_name)
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 JOIN albums alb ON alb.id = aa.album_id
                 WHERE alb.series_id = s.id), '') as author,
                s.description, s.cover_url, s.year, s.number_of_albums,
                s.bubble_id, s.slug, s.is_terminated, s.created_at
         FROM series s WHERE s.bubble_id = ?",
    )
    .bind(&bubble_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    {
        tracing::info!(%bubble_id, elapsed = ?start.elapsed(), "get_or_create_series: found in DB");
        return Ok(existing);
    }

    use crate::server::slug::generate_unique_slug;

    let provider = default_provider();
    let t = std::time::Instant::now();
    let (meta, albums) = provider.fetch_series_detail(&bubble_id).await?;
    tracing::info!(%bubble_id, album_count = albums.len(), elapsed = ?t.elapsed(), "fetch_series_detail");

    let slug = generate_unique_slug(&pool, "series", &meta.title)
        .await
        .map_err(|e| ServerFnError::new(format!("Slug generation error: {e}")))?;

    let series = sqlx::query_as::<_, Series>(
        "INSERT INTO series (title, work_type, description, cover_url, year, number_of_albums, bubble_id, slug)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING id, title, work_type,
           '' as author,
           description, cover_url, year, number_of_albums, bubble_id, slug, is_terminated, created_at",
    )
    .bind(&meta.title)
    .bind(&meta.work_type)
    .bind(&meta.description)
    .bind(&meta.cover_url)
    .bind(meta.year)
    .bind(meta.number_of_albums)
    .bind(&bubble_id)
    .bind(&slug)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to save series: {e}")))?;

    // Insert all albums and fetch enrichment data
    let enrichment_start = std::time::Instant::now();
    let mut enriched_count = 0u32;
    for album in &albums {
        let effective_title = album.title.as_deref().unwrap_or("");
        let album_slug_title = if effective_title.is_empty() {
            match album.tome {
                Some(n) => format!("{} tome {n}", &meta.title),
                None => meta.title.clone(),
            }
        } else {
            effective_title.to_string()
        };
        let album_slug = generate_unique_slug(&pool, "albums", &album_slug_title)
            .await
            .map_err(|e| ServerFnError::new(format!("Slug generation error: {e}")))?;

        let result = sqlx::query_scalar::<_, i64>(
            "INSERT OR IGNORE INTO albums (series_id, title, tome, cover_url, ean, bubble_id, slug)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(series.id)
        .bind(&album.title)
        .bind(album.tome)
        .bind(&album.cover_url)
        .bind(&album.ean)
        .bind(&album.provider_id)
        .bind(&album_slug)
        .fetch_optional(&pool)
        .await;

        let album_db_id = match result {
            Ok(Some(id)) => id,
            _ => continue, // already existed or error
        };

        // Fetch and store enrichment data
        let t = std::time::Instant::now();
        if let Ok(Some(enrichment)) = provider.fetch_album_enrichment(&album.provider_id).await {
            enriched_count += 1;
            tracing::debug!(provider_id = %album.provider_id, elapsed = ?t.elapsed(), "fetch_album_enrichment");
            sqlx::query(
                "UPDATE albums SET summary = ?, publisher = ?, number_of_pages = ?, publication_date = ?
                 WHERE id = ?",
            )
            .bind(&enrichment.summary)
            .bind(&enrichment.publisher)
            .bind(enrichment.number_of_pages)
            .bind(&enrichment.publication_date)
            .bind(album_db_id)
            .execute(&pool)
            .await
            .ok();

            // Update cover_url if enrichment provides a better one
            if let Some(cover) = &enrichment.cover_url {
                sqlx::query("UPDATE albums SET cover_url = ? WHERE id = ?")
                    .bind(cover)
                    .bind(album_db_id)
                    .execute(&pool)
                    .await
                    .ok();
            }

            for author in &enrichment.authors {
                // Upsert into authors table
                let author_id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO authors (display_name, slug, date_birth, date_death)
                     VALUES (?, ?, ?, ?)
                     ON CONFLICT(slug) WHERE slug IS NOT NULL DO UPDATE SET display_name = excluded.display_name
                     RETURNING id",
                )
                .bind(&author.display_name)
                .bind(&author.slug)
                .bind(&author.date_birth)
                .bind(&author.date_death)
                .fetch_one(&pool)
                .await;

                if let Ok(author_id) = author_id {
                    sqlx::query(
                        "INSERT OR IGNORE INTO album_authors (album_id, author_id, role)
                         VALUES (?, ?, ?)",
                    )
                    .bind(album_db_id)
                    .bind(author_id)
                    .bind(&author.role)
                    .execute(&pool)
                    .await
                    .ok();
                }
            }
        }
    }
    tracing::info!(
        %bubble_id,
        enriched_count,
        enrichment_elapsed = ?enrichment_start.elapsed(),
        total_elapsed = ?start.elapsed(),
        "get_or_create_series: done"
    );

    Ok(series)
}

/// Fetch a single series by slug.
#[server]
pub async fn get_series(slug: String) -> Result<Option<Series>, ServerFnError> {
    tracing::debug!(%slug, "get_series");
    let pool = crate::db::pool();

    let series = sqlx::query_as::<_, Series>(
        "SELECT s.id, s.title, s.work_type,
                COALESCE((SELECT GROUP_CONCAT(DISTINCT au.display_name)
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 JOIN albums alb ON alb.id = aa.album_id
                 WHERE alb.series_id = s.id), '') as author,
                s.description, s.cover_url, s.year, s.number_of_albums,
                s.bubble_id, s.slug, s.is_terminated, s.created_at
         FROM series s WHERE s.slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(series)
}

/// Get distinct authors for a series (with slugs and roles).
#[server]
pub async fn get_series_authors(series_id: i64) -> Result<Vec<AlbumAuthor>, ServerFnError> {
    let pool = crate::db::pool();

    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT DISTINCT au.display_name, aa.role, au.slug, au.date_birth, au.date_death
         FROM album_authors aa
         JOIN authors au ON au.id = aa.author_id
         JOIN albums alb ON alb.id = aa.album_id
         WHERE alb.series_id = ?
         ORDER BY au.display_name ASC",
    )
    .bind(series_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(rows
        .into_iter()
        .map(
            |(display_name, role, slug, date_birth, date_death)| AlbumAuthor {
                display_name,
                role,
                slug,
                date_birth,
                date_death,
            },
        )
        .collect())
}

/// Get all albums for a series with per-user ownership flags.
#[server]
pub async fn get_series_albums(series_id: i64) -> Result<Vec<AlbumWithOwnership>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user_id = get_current_user().await?.map(|u| u.id).unwrap_or(-1); // -1 means no user, LEFT JOIN will produce owned=false

    let albums = sqlx::query_as::<_, AlbumWithOwnership>(
        "SELECT a.id, a.series_id, a.title, a.tome, a.cover_url, a.ean,
                a.bubble_id, a.slug, a.created_at,
                CASE WHEN ua.album_id IS NOT NULL AND ua.owned = 1 THEN 1 ELSE 0 END AS owned,
                CASE WHEN al_borrow.album_id IS NOT NULL THEN 1 ELSE 0 END AS borrowed,
                CASE WHEN al_lend.album_id IS NOT NULL THEN 1 ELSE 0 END AS lent,
                COALESCE(ua.wishlisted, 0) AS wishlisted,
                ua.for_sale_price
         FROM albums a
         LEFT JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = ?
         LEFT JOIN album_loans al_borrow ON al_borrow.album_id = a.id AND al_borrow.borrower_id = ?
         LEFT JOIN album_loans al_lend ON al_lend.album_id = a.id AND al_lend.lender_id = ?
         WHERE a.series_id = ?
         ORDER BY a.tome ASC NULLS LAST, a.id ASC",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(series_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(albums)
}

/// Get albums for a series showing ownership for a specific public user (read-only).
#[server]
pub async fn get_series_albums_for_user(
    series_id: i64,
    username: String,
) -> Result<Vec<AlbumWithOwnership>, ServerFnError> {
    let pool = crate::db::pool();

    let albums = sqlx::query_as::<_, AlbumWithOwnership>(
        "SELECT a.id, a.series_id, a.title, a.tome, a.cover_url, a.ean,
                a.bubble_id, a.slug, a.created_at,
                CASE WHEN ua.album_id IS NOT NULL AND ua.owned = 1 THEN 1 ELSE 0 END AS owned,
                0 AS borrowed,
                0 AS lent,
                CASE WHEN u.wishlist_public = 1 THEN COALESCE(ua.wishlisted, 0) ELSE 0 END AS wishlisted,
                CASE WHEN ua.owned = 1 THEN ua.for_sale_price ELSE NULL END AS for_sale_price
         FROM albums a
         LEFT JOIN users u ON u.username = ? AND u.is_public = 1
         LEFT JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = u.id
         WHERE a.series_id = ?
         ORDER BY a.tome ASC NULLS LAST, a.id ASC",
    )
    .bind(&username)
    .bind(series_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(albums)
}

/// Set all albums in a series as owned or un-owned for the current user.
#[server]
pub async fn set_all_albums_owned(series_id: i64, owned: bool) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    if owned {
        // Insert new rows for albums not yet in user_albums
        sqlx::query(
            "INSERT OR IGNORE INTO user_albums (user_id, album_id, owned)
             SELECT ?, a.id, 1 FROM albums a WHERE a.series_id = ?",
        )
        .bind(user.id)
        .bind(series_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
        // Update existing rows: set owned, clear wishlist
        sqlx::query(
            "UPDATE user_albums SET owned = 1, wishlisted = 0
             WHERE user_id = ? AND album_id IN (SELECT id FROM albums WHERE series_id = ?)",
        )
        .bind(user.id)
        .bind(series_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    } else {
        // Clear for_sale_price and mark un-owned
        sqlx::query(
            "UPDATE user_albums SET owned = 0, for_sale_price = NULL
             WHERE user_id = ? AND album_id IN (SELECT id FROM albums WHERE series_id = ?)",
        )
        .bind(user.id)
        .bind(series_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
        // Delete rows that are neither owned nor wishlisted
        sqlx::query(
            "DELETE FROM user_albums
             WHERE user_id = ? AND owned = 0 AND wishlisted = 0
             AND album_id IN (SELECT id FROM albums WHERE series_id = ?)",
        )
        .bind(user.id)
        .bind(series_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
    }

    Ok(())
}

/// Toggle album ownership for the current user.
/// Returns the new owned state (true = now owned, false = now un-owned).
#[server]
pub async fn toggle_album_owned(album_id: i64) -> Result<bool, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    // Check current state
    let row = sqlx::query_as::<_, (bool, bool)>(
        "SELECT owned, wishlisted FROM user_albums WHERE user_id = ? AND album_id = ?",
    )
    .bind(user.id)
    .bind(album_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    match row {
        Some((true, _)) => {
            // Currently owned — un-own it
            sqlx::query(
                "UPDATE user_albums SET owned = 0, for_sale_price = NULL
                 WHERE user_id = ? AND album_id = ?",
            )
            .bind(user.id)
            .bind(album_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            // Clean up rows that are neither owned nor wishlisted
            sqlx::query(
                "DELETE FROM user_albums
                 WHERE user_id = ? AND album_id = ? AND owned = 0 AND wishlisted = 0",
            )
            .bind(user.id)
            .bind(album_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(false)
        }
        Some((false, _)) => {
            // Row exists but not owned (wishlisted) — own it, clear wishlist
            sqlx::query(
                "UPDATE user_albums SET owned = 1, wishlisted = 0
                 WHERE user_id = ? AND album_id = ?",
            )
            .bind(user.id)
            .bind(album_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(true)
        }
        None => {
            // No row — insert as owned
            sqlx::query("INSERT INTO user_albums (user_id, album_id, owned) VALUES (?, ?, 1)")
                .bind(user.id)
                .bind(album_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(true)
        }
    }
}

/// Toggle wishlist status for an album the user does not own.
/// Returns the new wishlisted state.
#[server]
pub async fn toggle_album_wishlisted(album_id: i64) -> Result<bool, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    // Check current state
    let row = sqlx::query_as::<_, (bool, bool)>(
        "SELECT owned, wishlisted FROM user_albums WHERE user_id = ? AND album_id = ?",
    )
    .bind(user.id)
    .bind(album_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    match row {
        Some((true, _)) => Err(ServerFnError::new(
            "Cannot wishlist an album you already own",
        )),
        Some((false, true)) => {
            // Currently wishlisted — remove (delete the row since owned=false)
            sqlx::query("DELETE FROM user_albums WHERE user_id = ? AND album_id = ?")
                .bind(user.id)
                .bind(album_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(false)
        }
        Some((false, false)) => {
            // Row exists, not owned, not wishlisted — set wishlisted
            sqlx::query("UPDATE user_albums SET wishlisted = 1 WHERE user_id = ? AND album_id = ?")
                .bind(user.id)
                .bind(album_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(true)
        }
        None => {
            // No row — insert as wishlisted
            sqlx::query(
                "INSERT INTO user_albums (user_id, album_id, owned, wishlisted) VALUES (?, ?, 0, 1)",
            )
            .bind(user.id)
            .bind(album_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;
            Ok(true)
        }
    }
}

/// Set or clear the for-sale price on an owned album.
/// Pass `None` to remove the listing.
#[server]
pub async fn set_album_for_sale(album_id: i64, price: Option<f64>) -> Result<(), ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    // Verify the user owns this album
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT owned FROM user_albums WHERE user_id = ? AND album_id = ?",
    )
    .bind(user.id)
    .bind(album_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if owned != Some(true) {
        return Err(ServerFnError::new(
            "Cannot mark an album for sale that you don't own",
        ));
    }

    sqlx::query("UPDATE user_albums SET for_sale_price = ? WHERE user_id = ? AND album_id = ?")
        .bind(price)
        .bind(user.id)
        .bind(album_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(())
}

/// Get the current user's wishlisted albums.
#[server]
pub async fn get_user_wishlist() -> Result<Vec<crate::models::WishlistItem>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let items = sqlx::query_as::<_, crate::models::WishlistItem>(
        "SELECT a.id AS album_id, a.title AS album_title, a.slug AS album_slug,
                s.title AS series_title, a.tome, a.cover_url
         FROM user_albums ua
         JOIN albums a ON a.id = ua.album_id
         JOIN series s ON s.id = a.series_id
         WHERE ua.user_id = ? AND ua.wishlisted = 1
         ORDER BY s.title ASC, a.tome ASC NULLS LAST",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}

/// Search the current user's collection by title or author.
#[server]
pub async fn search_user_collection(
    query: String,
) -> Result<Vec<SeriesWithOwnership>, ServerFnError> {
    use crate::server::auth::get_current_user;
    tracing::debug!(%query, "search_user_collection");
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(vec![]),
    };

    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let pattern = format!("%{}%", query.trim());

    let items = sqlx::query_as::<_, SeriesWithOwnership>(
        "SELECT s.id, s.title, s.work_type,
                COALESCE((SELECT GROUP_CONCAT(DISTINCT au.display_name)
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 JOIN albums alb ON alb.id = aa.album_id
                 WHERE alb.series_id = s.id), '') as author,
                s.description, s.cover_url, s.year, s.number_of_albums,
                s.bubble_id, s.slug, s.is_terminated, s.created_at,
                COUNT(DISTINCT CASE WHEN ua.owned = 1 THEN ua.album_id END) as owned_count,
                (SELECT COUNT(*) FROM albums WHERE series_id = s.id) as total_albums,
                COUNT(DISTINCT CASE WHEN ua.owned = 1 AND ua.for_sale_price IS NOT NULL THEN ua.album_id END) as for_sale_count
         FROM series s
         JOIN albums a ON a.series_id = s.id
         LEFT JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = ?
         LEFT JOIN album_loans al ON al.album_id = a.id AND al.borrower_id = ?
         WHERE ((ua.album_id IS NOT NULL AND ua.owned = 1) OR al.album_id IS NOT NULL)
           AND (s.title LIKE ? OR EXISTS (
             SELECT 1 FROM album_authors aa2
             JOIN authors au2 ON au2.id = aa2.author_id
             JOIN albums alb2 ON alb2.id = aa2.album_id
             WHERE alb2.series_id = s.id AND au2.display_name LIKE ?
           ))
         GROUP BY s.id
         ORDER BY s.title ASC",
    )
    .bind(user.id)
    .bind(user.id)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}

/// Get the total number of albums owned by the current user.
#[server]
pub async fn get_user_album_count() -> Result<i64, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(0),
    };

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_albums WHERE user_id = ? AND owned = 1")
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(count)
}

/// Get the current user's collection: series with at least one owned album.
#[server]
pub async fn get_user_collection(
    sort_by: Option<String>,
) -> Result<Vec<SeriesWithOwnership>, ServerFnError> {
    use crate::server::auth::get_current_user;
    tracing::debug!(?sort_by, "get_user_collection");
    let pool = crate::db::pool();
    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let order = match sort_by.as_deref() {
        Some("title") => "ORDER BY s.title ASC",
        _ => "ORDER BY MAX(ua.owned_at) DESC",
    };

    let sql = format!(
        "SELECT s.id, s.title, s.work_type,
                COALESCE((SELECT GROUP_CONCAT(DISTINCT au.display_name)
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 JOIN albums alb ON alb.id = aa.album_id
                 WHERE alb.series_id = s.id), '') as author,
                s.description, s.cover_url, s.year, s.number_of_albums,
                s.bubble_id, s.slug, s.is_terminated, s.created_at,
                COUNT(DISTINCT CASE WHEN ua.owned = 1 THEN ua.album_id END) as owned_count,
                (SELECT COUNT(*) FROM albums WHERE series_id = s.id) as total_albums,
                COUNT(DISTINCT CASE WHEN ua.owned = 1 AND ua.for_sale_price IS NOT NULL THEN ua.album_id END) as for_sale_count
         FROM series s
         JOIN albums a ON a.series_id = s.id
         LEFT JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = ?
         LEFT JOIN album_loans al ON al.album_id = a.id AND al.borrower_id = ?
         WHERE (ua.album_id IS NOT NULL AND ua.owned = 1) OR al.album_id IS NOT NULL
         GROUP BY s.id
         {order}"
    );

    let items = sqlx::query_as::<_, SeriesWithOwnership>(&sql)
        .bind(user.id)
        .bind(user.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}

/// Get full album details: DB data + series info + optional external provider enrichment.
#[server]
pub async fn get_album_detail(slug: String) -> Result<Option<AlbumDetail>, ServerFnError> {
    use crate::server::auth::get_current_user;
    tracing::debug!(%slug, "get_album_detail");
    let pool = crate::db::pool();

    let user_id = get_current_user().await?.map(|u| u.id).unwrap_or(-1);

    // Fetch album + series info + ownership in one query
    let row = sqlx::query_as::<
        _,
        (
            i64,            // a.id
            i64,            // a.series_id
            Option<String>, // a.title
            Option<i32>,    // a.tome
            Option<String>, // a.cover_url
            Option<String>, // a.ean
            Option<String>, // a.summary
            Option<String>, // a.publisher
            Option<i64>,    // a.number_of_pages
            Option<String>, // a.publication_date
            String,         // a.slug
            String,         // s.title
            String,         // s.slug
            bool,           // owned
            bool,           // wishlisted
            Option<f64>,    // for_sale_price
        ),
    >(
        "SELECT a.id, a.series_id, a.title, a.tome, a.cover_url, a.ean,
                a.summary, a.publisher, a.number_of_pages, a.publication_date,
                a.slug,
                s.title, s.slug,
                CASE WHEN ua.album_id IS NOT NULL AND ua.owned = 1 THEN 1 ELSE 0 END AS owned,
                COALESCE(ua.wishlisted, 0) AS wishlisted,
                ua.for_sale_price
         FROM albums a
         JOIN series s ON s.id = a.series_id
         LEFT JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = ?
         WHERE a.slug = ?",
    )
    .bind(user_id)
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    let Some((
        id,
        series_id,
        title,
        tome,
        cover_url,
        ean,
        summary,
        publisher,
        number_of_pages,
        publication_date,
        album_slug,
        series_title,
        series_slug,
        owned,
        wishlisted,
        for_sale_price,
    )) = row
    else {
        return Ok(None);
    };

    // Fetch authors and tags from DB
    let authors = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT au.display_name, aa.role, au.slug, au.date_birth, au.date_death
         FROM album_authors aa
         JOIN authors au ON au.id = aa.author_id
         WHERE aa.album_id = ?",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(
        |(display_name, role, slug, date_birth, date_death)| crate::models::AlbumAuthor {
            display_name,
            role,
            slug,
            date_birth,
            date_death,
        },
    )
    .collect();

    let mut detail = AlbumDetail {
        id,
        series_id,
        title,
        tome,
        cover_url,
        ean,
        slug: album_slug,
        owned,
        wishlisted,
        for_sale_price,
        series_title,
        series_slug,
        summary,
        publisher,
        number_of_pages,
        publication_date,
        authors,
        lent_to: None,
        borrowed_from: None,
    };

    // Check if current user has lent this album
    if user_id > 0
        && let Ok(Some(row)) = sqlx::query_as::<_, (i64, String)>(
            "SELECT al.borrower_id, u.display_name
             FROM album_loans al
             JOIN users u ON u.id = al.borrower_id
             WHERE al.album_id = ? AND al.lender_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
    {
        detail.lent_to = Some(row);
    }

    // Check if current user is borrowing this album
    if user_id > 0
        && let Ok(Some(row)) = sqlx::query_as::<_, (i64, String)>(
            "SELECT al.lender_id, u.display_name
             FROM album_loans al
             JOIN users u ON u.id = al.lender_id
             WHERE al.album_id = ? AND al.borrower_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
    {
        detail.borrowed_from = Some(row);
    }

    Ok(Some(detail))
}

/// Search the current user's collection for series by a given author slug.
#[server]
pub async fn search_series_by_author(
    author_slug: String,
) -> Result<Vec<SeriesWithOwnership>, ServerFnError> {
    use crate::server::auth::get_current_user;
    let pool = crate::db::pool();

    let user = match get_current_user().await? {
        Some(u) => u,
        None => return Ok(vec![]),
    };

    if author_slug.trim().is_empty() {
        return Ok(vec![]);
    }

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
         JOIN user_albums ua ON ua.album_id = a.id AND ua.user_id = ? AND ua.owned = 1
         WHERE EXISTS (
             SELECT 1 FROM album_authors aa2
             JOIN authors au2 ON au2.id = aa2.author_id
             JOIN albums alb2 ON alb2.id = aa2.album_id
             WHERE alb2.series_id = s.id AND au2.slug = ?
         )
         GROUP BY s.id
         ORDER BY s.title ASC",
    )
    .bind(user.id)
    .bind(&author_slug)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(items)
}
