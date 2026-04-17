use crate::models::{Author, AuthorInfo, Series};
use leptos::prelude::*;
use server_fn::ServerFnError;

/// Search for series by title using the default metadata provider.
#[server]
pub async fn search_series(query: String) -> Result<Vec<Series>, ServerFnError> {
    use crate::server::provider::{MetadataProvider, default_provider};

    tracing::debug!(%query, "search_series");
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    default_provider().search_series(&query).await
}

/// Search for series by EAN barcode using the default metadata provider.
#[server]
pub async fn search_by_ean(ean: String) -> Result<Vec<Series>, ServerFnError> {
    use crate::server::provider::{MetadataProvider, default_provider};

    tracing::debug!(%ean, "search_by_ean");
    if ean.trim().is_empty() {
        return Ok(vec![]);
    }

    default_provider().search_by_ean(&ean).await
}

/// Resolve series + mark a scanned album as owned. Used by barcode flow.
/// Returns the series slug for redirect.
#[server]
pub async fn add_album_by_ean(
    ean: String,
    series_bubble_id: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::get_current_user;
    use crate::server::series::get_or_create_series;

    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let series = get_or_create_series(series_bubble_id).await?;

    // Find the album by EAN
    let pool = crate::db::pool();
    if let Some(album_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM albums WHERE series_id = ? AND ean = ?")
            .bind(series.id)
            .bind(&ean)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    {
        // Mark as owned (ignore if already owned)
        sqlx::query("INSERT OR IGNORE INTO user_albums (user_id, album_id) VALUES (?, ?)")
            .bind(user.id)
            .bind(album_id)
            .execute(&pool)
            .await
            .ok();
    }

    Ok(series.slug)
}

/// Search for series by title, fetching all pages from the provider.
/// Used by the author page to get complete results.
#[server]
pub async fn search_series_all(query: String) -> Result<Vec<Series>, ServerFnError> {
    use crate::server::provider::{MetadataProvider, default_provider};

    tracing::debug!(%query, "search_series_all");
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    default_provider().search_series_all(&query).await
}

/// Search for authors using the default metadata provider. Returns up to 5 results.
#[server]
pub async fn search_authors_api(query: String) -> Result<Vec<AuthorInfo>, ServerFnError> {
    use crate::server::provider::{MetadataProvider, default_provider};

    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    default_provider().search_authors(&query).await
}

/// Look up an author from the local database by slug.
#[server]
pub async fn get_author_by_slug(slug: String) -> Result<Option<Author>, ServerFnError> {
    let pool = crate::db::pool();
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT display_name, slug, bio, date_birth, date_death FROM authors WHERE slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(
        row.map(|(display_name, slug, bio, date_birth, date_death)| Author {
            display_name,
            slug,
            bio,
            date_birth,
            date_death,
        }),
    )
}
