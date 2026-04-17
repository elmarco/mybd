use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Utility functions (replicated from main app to avoid Leptos dependency)
// ---------------------------------------------------------------------------

pub(crate) fn slugify(input: &str) -> String {
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

async fn generate_unique_slug(pool: &SqlitePool, table: &str, title: &str) -> Result<String> {
    let base = slugify(title);
    let pattern = format!("{base}%");
    let existing: Vec<String> =
        sqlx::query_scalar(&format!("SELECT slug FROM {table} WHERE slug LIKE ?"))
            .bind(&pattern)
            .fetch_all(pool)
            .await?;

    if !existing.contains(&base) {
        return Ok(base);
    }

    let mut n = 2u32;
    loop {
        let candidate = format!("{base}-{n}");
        if !existing.contains(&candidate) {
            return Ok(candidate);
        }
        n += 1;
    }
}

fn gravatar_url(email: &str, size: u32) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(email.trim().to_lowercase().as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("https://gravatar.com/avatar/{hex}?d=identicon&s={size}")
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

async fn create_user(
    pool: &SqlitePool,
    username: &str,
    email: &str,
    password: &str,
    display_name: &str,
    is_public: bool,
) -> Result<i64> {
    use argon2::{
        Argon2, PasswordHasher, password_hash::SaltString, password_hash::rand_core::OsRng,
    };

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {e}"))?
        .to_string();

    let avatar_url = gravatar_url(email, 200);

    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, email, display_name, password_hash, avatar_url, is_public)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(username)
    .bind(email)
    .bind(display_name)
    .bind(&password_hash)
    .bind(&avatar_url)
    .bind(is_public)
    .fetch_one(pool)
    .await
    .with_context(|| format!("Failed to create user {username}"))?;

    println!("  Created user: {display_name} (@{username}, public={is_public})");
    Ok(user_id)
}

async fn add_follow(pool: &SqlitePool, user_id: i64, following_id: i64) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO follows (user_id, following_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(following_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Search for a series via Bubble BD, fetch full details, and import into the database.
/// Returns the local series ID.
async fn search_and_import_series(
    pool: &SqlitePool,
    client: &bubblebd::Client,
    query: &str,
) -> Result<i64> {
    println!("\n  Searching: \"{query}\"...");

    let hits = client
        .search_series(query)
        .await
        .with_context(|| format!("Search failed for \"{query}\""))?;

    let hit = hits
        .first()
        .with_context(|| format!("No results found for \"{query}\""))?;

    println!("  Found: {} (bubble_id={})", hit.title, hit.object_id);

    // Fetch full series detail from the REST API
    let (api_series, api_albums) = client
        .get_series(&hit.object_id)
        .await
        .with_context(|| format!("Failed to fetch series detail for {}", hit.object_id))?;

    println!(
        "  Series: {} ({} albums)",
        api_series.title,
        api_albums.len()
    );

    // Insert series
    let slug = generate_unique_slug(pool, "series", &api_series.title).await?;

    let series_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO series (title, work_type, description, cover_url, year, number_of_albums, bubble_id, slug, is_terminated)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(&api_series.title)
    .bind(api_series.work_type.to_string())
    .bind(&api_series.description)
    .bind(&api_series.cover_url)
    .bind(api_series.year)
    .bind(api_series.number_of_albums.map(|n| n as i32))
    .bind(&hit.object_id)
    .bind(&slug)
    .bind(api_series.is_terminated)
    .fetch_one(pool)
    .await
    .context("Failed to insert series")?;

    // Insert albums and fetch enrichment
    for album_info in &api_albums {
        let effective_title = album_info.title.as_deref().unwrap_or("");
        let album_slug_title = if effective_title.is_empty() {
            match album_info.tome {
                Some(n) => format!("{} tome {n}", &api_series.title),
                None => api_series.title.clone(),
            }
        } else {
            effective_title.to_string()
        };
        let album_slug = generate_unique_slug(pool, "albums", &album_slug_title).await?;

        let result = sqlx::query_scalar::<_, i64>(
            "INSERT OR IGNORE INTO albums (series_id, title, tome, cover_url, ean, bubble_id, slug)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(series_id)
        .bind(&album_info.title)
        .bind(album_info.tome.map(|t| t as i32))
        .bind(&album_info.cover_url)
        .bind(&album_info.ean)
        .bind(&album_info.object_id)
        .bind(&album_slug)
        .fetch_optional(pool)
        .await;

        let album_db_id = match result {
            Ok(Some(id)) => id,
            _ => continue,
        };

        // Fetch enrichment from album detail endpoint
        if let Ok(api_album) = client.get_album(&album_info.object_id).await {
            if let Some(summary) = &api_album.summary {
                sqlx::query("UPDATE albums SET summary = ? WHERE id = ?")
                    .bind(summary)
                    .bind(album_db_id)
                    .execute(pool)
                    .await
                    .ok();
            }

            if let Some(cover) = &api_album.cover_url {
                sqlx::query("UPDATE albums SET cover_url = ? WHERE id = ?")
                    .bind(cover)
                    .bind(album_db_id)
                    .execute(pool)
                    .await
                    .ok();
            }

            if let Some(print) = api_album.prints.first() {
                let publisher_name = print.publisher.as_ref().map(|p| &p.name);
                sqlx::query(
                    "UPDATE albums SET publisher = ?, number_of_pages = ?, publication_date = ?,
                            height_cm = ?, width_cm = ?, length_cm = ?, weight_kg = ?
                     WHERE id = ?",
                )
                .bind(publisher_name)
                .bind(print.number_of_pages)
                .bind(&print.publication_date)
                .bind(print.height_cm)
                .bind(print.width_cm)
                .bind(print.length_cm)
                .bind(print.weight_kg)
                .bind(album_db_id)
                .execute(pool)
                .await
                .ok();

                for author in &print.authors {
                    let slug = slugify(&author.display_name);
                    let author_db_id = sqlx::query_scalar::<_, i64>(
                        "INSERT INTO authors (display_name, bubble_id, slug, date_birth, date_death)
                         VALUES (?, ?, ?, ?, ?)
                         ON CONFLICT(bubble_id) DO UPDATE SET display_name = excluded.display_name
                         RETURNING id",
                    )
                    .bind(&author.display_name)
                    .bind(&author.object_id)
                    .bind(&slug)
                    .bind(&author.year_of_birth)
                    .bind(&author.year_of_death)
                    .fetch_one(pool)
                    .await;

                    if let Ok(author_db_id) = author_db_id {
                        sqlx::query(
                            "INSERT OR IGNORE INTO album_authors (album_id, author_id, role)
                             VALUES (?, ?, ?)",
                        )
                        .bind(album_db_id)
                        .bind(author_db_id)
                        .bind(&author.role)
                        .execute(pool)
                        .await
                        .ok();
                    }
                }
            }
        }

        let tome_str = album_info
            .tome
            .map(|t| format!(" T{t}"))
            .unwrap_or_default();
        println!("    Album{tome_str}: {album_slug}");
    }

    Ok(series_id)
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

pub async fn run(db_url: &str, force: bool) -> Result<()> {
    // 1. Delete existing DB file and create fresh
    let db_path = db_url.strip_prefix("sqlite:").unwrap_or(db_url);
    if std::path::Path::new(db_path).exists() {
        if !force {
            anyhow::bail!("Database {db_path} already exists. Use -f/--force to overwrite.");
        }
        std::fs::remove_file(db_path).with_context(|| format!("Failed to remove {db_path}"))?;
        println!("Removed existing database: {db_path}");
    }

    let options = SqliteConnectOptions::from_str(db_url)
        .context("Invalid database URL")?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    println!("Created fresh database: {db_path}");

    // 2. Create users
    println!("\nCreating users...");
    let marc_id = create_user(&pool, "marc", "marc@test.com", "password123", "Marc", true).await?;
    let tom_id = create_user(&pool, "tom", "tom@test.com", "password123", "Tom", true).await?;
    let _luc_id = create_user(&pool, "luc", "luc@test.com", "password123", "Luc", false).await?;

    // 3. Follows: Marc <-> Tom (mutual follows)
    println!("\nCreating follows...");
    add_follow(&pool, marc_id, tom_id).await?;
    add_follow(&pool, tom_id, marc_id).await?;
    println!("  Marc <-> Tom now follow each other");

    // 4. Search and import series using the Bubble BD backend
    println!("\nImporting series from Bubble BD...");
    let client = bubblebd::Client::new();

    let _futurs_id = search_and_import_series(&pool, &client, "Les Futurs de Liu Cixin").await?;
    let terre_id = search_and_import_series(&pool, &client, "Terre ou Lune").await?;

    // 5. Marc owns all albums
    println!("\nAssigning ownership...");
    let count = sqlx::query_scalar::<_, i64>(
        "INSERT INTO user_albums (user_id, album_id)
         SELECT ?, id FROM albums
         RETURNING album_id",
    )
    .bind(marc_id)
    .fetch_all(&pool)
    .await?
    .len();
    println!("  Marc now owns {count} albums");

    // 6. Marc lends all "Terre ou Lune" albums to Tom
    println!("\nCreating loans...");
    let lent_albums: Vec<(i64, Option<String>, Option<i64>)> =
        sqlx::query_as("SELECT a.id, a.title, a.tome FROM albums a WHERE a.series_id = ?")
            .bind(terre_id)
            .fetch_all(&pool)
            .await?;

    for (album_id, title, tome) in &lent_albums {
        sqlx::query("INSERT INTO album_loans (lender_id, borrower_id, album_id) VALUES (?, ?, ?)")
            .bind(marc_id)
            .bind(tom_id)
            .bind(album_id)
            .execute(&pool)
            .await
            .with_context(|| format!("Failed to create loan for album {album_id}"))?;
        let display = match (title.as_deref(), tome) {
            (Some(t), _) => t.to_string(),
            (None, Some(n)) => format!("Terre ou Lune T{n}"),
            (None, None) => "Terre ou Lune".to_string(),
        };
        println!("  Marc lent \"{display}\" to Tom");
    }

    println!("\nDatabase populated successfully at {db_path}!");
    Ok(())
}
