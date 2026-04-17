/// Convert a string to a URL-friendly slug.
///
/// - Unicode NFD decomposition, strip combining marks (accents)
/// - Lowercase
/// - Replace non-alphanumeric with `-`
/// - Collapse consecutive `-`, trim leading/trailing `-`
/// - Return `"untitled"` if empty
pub fn slugify(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;

    let slug: String = input
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

#[cfg(feature = "ssr")]
/// Generate a unique slug for the given table.
///
/// Queries the DB for existing slugs starting with the base slug,
/// appending `-2`, `-3`, etc. if the base is taken.
pub async fn generate_unique_slug(
    pool: &sqlx::SqlitePool,
    table: &str,
    title: &str,
) -> Result<String, sqlx::Error> {
    let base = slugify(title);

    // Check if base slug is free
    let pattern = format!("{base}%");
    let existing: Vec<String> =
        sqlx::query_scalar(&format!("SELECT slug FROM {table} WHERE slug LIKE ?"))
            .bind(&pattern)
            .fetch_all(pool)
            .await?;

    if !existing.contains(&base) {
        return Ok(base);
    }

    // Find next available suffix
    let mut n = 2u32;
    loop {
        let candidate = format!("{base}-{n}");
        if !existing.contains(&candidate) {
            return Ok(candidate);
        }
        n += 1;
    }
}

#[cfg(feature = "ssr")]
/// Backfill slugs for all series and albums with NULL slug.
/// Creates unique indexes after all rows have slugs.
pub async fn backfill_slugs(pool: &sqlx::SqlitePool) {
    // Backfill series slugs
    let series_rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, title FROM series WHERE slug IS NULL")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    for (id, title) in &series_rows {
        match generate_unique_slug(pool, "series", title).await {
            Ok(slug) => {
                if let Err(e) = sqlx::query("UPDATE series SET slug = ? WHERE id = ?")
                    .bind(&slug)
                    .bind(id)
                    .execute(pool)
                    .await
                {
                    tracing::warn!("failed to set slug for series {id}: {e}");
                }
            }
            Err(e) => tracing::warn!("failed to generate slug for series {id} ({title}): {e}"),
        }
    }

    // Backfill album slugs — need series slug for fallback titles
    let album_rows: Vec<(i64, Option<String>, Option<i32>, i64)> = sqlx::query_as(
        "SELECT a.id, a.title, a.tome, a.series_id FROM albums a WHERE a.slug IS NULL",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (id, title, tome, series_id) in &album_rows {
        let series_slug: String = sqlx::query_scalar("SELECT slug FROM series WHERE id = ?")
            .bind(series_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| format!("series-{series_id}"));

        let effective_title = match title {
            Some(t) if !t.is_empty() => t.clone(),
            _ => match tome {
                Some(n) => format!("{series_slug} tome {n}"),
                None => format!("{series_slug} album {id}"),
            },
        };

        match generate_unique_slug(pool, "albums", &effective_title).await {
            Ok(slug) => {
                if let Err(e) = sqlx::query("UPDATE albums SET slug = ? WHERE id = ?")
                    .bind(&slug)
                    .bind(id)
                    .execute(pool)
                    .await
                {
                    tracing::warn!("failed to set slug for album {id}: {e}");
                }
            }
            Err(e) => {
                tracing::warn!(
                    "failed to generate slug for album {id} ({effective_title}): {e}"
                );
            }
        }
    }

    // Create unique indexes now that all rows have slugs
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_series_slug ON series(slug)")
        .execute(pool)
        .await
        .ok();
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_albums_slug ON albums(slug)")
        .execute(pool)
        .await
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_title() {
        assert_eq!(slugify("Donjon Zénith"), "donjon-zenith");
    }

    #[test]
    fn strips_special_chars() {
        assert_eq!(slugify("One Piece: L'Aube!"), "one-piece-l-aube");
    }

    #[test]
    fn collapses_dashes() {
        assert_eq!(slugify("a---b"), "a-b");
    }

    #[test]
    fn trims_dashes() {
        assert_eq!(slugify("--hello--"), "hello");
    }

    #[test]
    fn empty_input() {
        assert_eq!(slugify(""), "untitled");
    }

    #[test]
    fn only_special_chars() {
        assert_eq!(slugify("---!!!---"), "untitled");
    }

    #[test]
    fn japanese_characters() {
        // CJK chars pass through as-is (no combining marks to strip)
        let result = slugify("ワンピース");
        assert!(!result.is_empty());
        assert_ne!(result, "untitled");
    }
}
