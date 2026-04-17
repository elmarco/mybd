mod models;
mod populate;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use models::{AlbumAuthorToml, AlbumToml, AuthorToml, SeriesToml};
use sqlx::sqlite::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "sqlite:mybd.db")]
    db: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Export database records to TOML files
    Export {
        #[arg(short, long, default_value = "data/series")]
        out_dir: PathBuf,
    },
    /// Import database records from TOML files
    Import {
        #[arg(short, long, default_value = "data/series")]
        in_dir: PathBuf,
        /// Delete DB rows whose TOML files have been removed
        #[arg(long)]
        delete: bool,
    },
    /// Create a fresh test database with sample users, series, and loans
    Populate {
        /// Overwrite existing database without prompting
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Export { out_dir } => {
            let pool = SqlitePool::connect(&cli.db)
                .await
                .with_context(|| format!("Failed to connect to database at {}", cli.db))?;
            run_migrations(&pool).await?;
            export_data(&pool, &out_dir).await?;
        }
        Commands::Import { in_dir, delete } => {
            let pool = SqlitePool::connect(&cli.db)
                .await
                .with_context(|| format!("Failed to connect to database at {}", cli.db))?;
            run_migrations(&pool).await?;
            import_data(&pool, &in_dir, delete).await?;
        }
        Commands::Populate { force } => {
            populate::run(&cli.db, force).await?;
        }
    }

    Ok(())
}

async fn export_data(pool: &SqlitePool, out_dir: &Path) -> Result<()> {
    fs::create_dir_all(out_dir).context("Failed to create export directory")?;

    let series_rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<String>, Option<String>, Option<bool>)>(
        "SELECT id, title, work_type, description, cover_url, year, number_of_albums, bubble_id, slug, is_terminated FROM series"
    )
    .fetch_all(pool)
    .await?;

    for (
        id,
        title,
        work_type,
        description,
        cover_url,
        year,
        number_of_albums,
        bubble_id,
        slug,
        is_terminated,
    ) in series_rows
    {
        let album_rows = sqlx::query_as::<
            _,
            (
                i64,
                Option<String>,
                Option<i64>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<String>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
            ),
        >(
            "SELECT id, title, tome, cover_url, ean, bubble_id, slug,
                    summary, publisher, number_of_pages, publication_date,
                    height_cm, width_cm, length_cm, weight_kg
             FROM albums WHERE series_id = ? ORDER BY tome ASC, title ASC",
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let mut albums = Vec::new();
        for (
            album_id,
            a_title,
            tome,
            a_cover_url,
            ean,
            a_bubble_id,
            a_slug,
            summary,
            publisher,
            number_of_pages,
            publication_date,
            height_cm,
            width_cm,
            length_cm,
            weight_kg,
        ) in album_rows
        {
            let authors = sqlx::query_as::<_, (String, Option<String>)>(
                "SELECT au.slug, aa.role
                 FROM album_authors aa
                 JOIN authors au ON au.id = aa.author_id
                 WHERE aa.album_id = ?",
            )
            .bind(album_id)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|(slug, role)| AlbumAuthorToml { slug, role })
            .collect::<Vec<_>>();

            albums.push(AlbumToml {
                title: a_title,
                tome,
                cover_url: a_cover_url,
                ean,
                bubble_id: a_bubble_id,
                slug: a_slug,
                summary,
                publisher,
                number_of_pages,
                publication_date,
                height_cm,
                width_cm,
                length_cm,
                weight_kg,
                authors,
            });
        }

        let series_toml = SeriesToml {
            title: title.clone(),
            work_type,
            description,
            cover_url,
            year,
            number_of_albums,
            bubble_id: bubble_id.clone(),
            slug: slug.clone(),
            is_terminated,
            albums,
        };

        let toml_string = toml::to_string_pretty(&series_toml)?;

        // Generate a safe filename
        let filename = if let Some(ref slug) = series_toml.slug {
            format!("{slug}.toml")
        } else if let Some(bid) = &series_toml.bubble_id {
            format!("bubble-{bid}.toml")
        } else {
            format!("series-{id}.toml")
        };

        let file_path = out_dir.join(filename);
        fs::write(&file_path, toml_string)
            .with_context(|| format!("Failed to write to file {:?}", file_path))?;
        println!("Exported: {:?}", file_path);
    }

    // Export authors
    let authors_dir = out_dir.parent().unwrap_or(out_dir).join("authors");
    fs::create_dir_all(&authors_dir)?;

    let author_rows =
        sqlx::query_as::<
            _,
            (
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            ),
        >("SELECT display_name, bubble_id, slug, bio, date_birth, date_death FROM authors")
        .fetch_all(pool)
        .await?;

    for (display_name, bubble_id, slug, bio, date_birth, date_death) in author_rows {
        let author = AuthorToml {
            display_name: display_name.clone(),
            bubble_id: bubble_id.clone(),
            slug: slug.clone(),
            bio,
            date_birth,
            date_death,
        };
        let filename =
            slug.unwrap_or_else(|| format!("author-{}", bubble_id.as_deref().unwrap_or("unknown")));
        let path = authors_dir.join(format!("{filename}.toml"));
        fs::write(&path, toml::to_string_pretty(&author)?)?;
        println!("Exported: {:?}", path);
    }

    Ok(())
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

async fn import_data(pool: &SqlitePool, in_dir: &Path, delete: bool) -> Result<()> {
    if !in_dir.exists() {
        return Err(anyhow::anyhow!(
            "Import directory {:?} does not exist",
            in_dir
        ));
    }

    let mut imported_author_slugs: Vec<String> = Vec::new();
    let mut imported_series_bids: Vec<String> = Vec::new();
    let mut imported_album_bids: Vec<String> = Vec::new();

    // Import authors first
    let authors_dir = in_dir.parent().unwrap_or(in_dir).join("authors");
    if authors_dir.exists() {
        for entry in WalkDir::new(&authors_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        {
            let content = fs::read_to_string(entry.path())?;
            let author: AuthorToml = toml::from_str(&content)?;
            if let Some(slug) = &author.slug {
                imported_author_slugs.push(slug.clone());
            }
            sqlx::query(
                "INSERT INTO authors (display_name, bubble_id, slug, bio, date_birth, date_death)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(bubble_id) WHERE bubble_id IS NOT NULL DO UPDATE SET
                     display_name = excluded.display_name,
                     slug = excluded.slug,
                     bio = excluded.bio,
                     date_birth = excluded.date_birth,
                     date_death = excluded.date_death",
            )
            .bind(&author.display_name)
            .bind(&author.bubble_id)
            .bind(&author.slug)
            .bind(&author.bio)
            .bind(&author.date_birth)
            .bind(&author.date_death)
            .execute(pool)
            .await?;
            println!("Imported author: {}", author.display_name);
        }
    }

    for entry in WalkDir::new(in_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
    {
        let path = entry.path();
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file {:?}", path))?;
        let series: SeriesToml = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML in {:?}", path))?;

        if let Some(bid) = &series.bubble_id {
            imported_series_bids.push(bid.clone());
        }
        for album in &series.albums {
            if let Some(bid) = &album.bubble_id {
                imported_album_bids.push(bid.clone());
            }
            for author in &album.authors {
                imported_author_slugs.push(author.slug.clone());
            }
        }

        let mut tx = pool.begin().await?;

        // Upsert Series using bubble_id as the unique key.
        let series_id: i64 = if series.bubble_id.is_some() {
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO series (title, work_type, description, cover_url, year, number_of_albums, bubble_id, slug, is_terminated)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(bubble_id) WHERE bubble_id IS NOT NULL DO UPDATE SET
                     title = excluded.title,
                     work_type = excluded.work_type,
                     description = excluded.description,
                     cover_url = excluded.cover_url,
                     year = excluded.year,
                     number_of_albums = excluded.number_of_albums,
                     is_terminated = excluded.is_terminated
                 RETURNING id"
            )
            .bind(&series.title)
            .bind(&series.work_type)
            .bind(&series.description)
            .bind(&series.cover_url)
            .bind(series.year)
            .bind(series.number_of_albums)
            .bind(&series.bubble_id)
            .bind(&series.slug)
            .bind(series.is_terminated)
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO series (title, work_type, description, cover_url, year, number_of_albums, slug, is_terminated)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 RETURNING id"
            )
            .bind(&series.title)
            .bind(&series.work_type)
            .bind(&series.description)
            .bind(&series.cover_url)
            .bind(series.year)
            .bind(series.number_of_albums)
            .bind(&series.slug)
            .bind(series.is_terminated)
            .fetch_one(&mut *tx)
            .await?
        };

        // Upsert Albums
        for album in &series.albums {
            let album_id: i64 = if album.bubble_id.is_some() {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO albums (series_id, title, tome, cover_url, ean, bubble_id, slug,
                                        summary, publisher, number_of_pages, publication_date,
                                        height_cm, width_cm, length_cm, weight_kg)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(bubble_id) WHERE bubble_id IS NOT NULL DO UPDATE SET
                         series_id = excluded.series_id,
                         title = excluded.title,
                         tome = excluded.tome,
                         cover_url = excluded.cover_url,
                         ean = excluded.ean,
                         summary = excluded.summary,
                         publisher = excluded.publisher,
                         number_of_pages = excluded.number_of_pages,
                         publication_date = excluded.publication_date,
                         height_cm = excluded.height_cm,
                         width_cm = excluded.width_cm,
                         length_cm = excluded.length_cm,
                         weight_kg = excluded.weight_kg
                     RETURNING id",
                )
                .bind(series_id)
                .bind(&album.title)
                .bind(album.tome)
                .bind(&album.cover_url)
                .bind(&album.ean)
                .bind(&album.bubble_id)
                .bind(&album.slug)
                .bind(&album.summary)
                .bind(&album.publisher)
                .bind(album.number_of_pages)
                .bind(&album.publication_date)
                .bind(album.height_cm)
                .bind(album.width_cm)
                .bind(album.length_cm)
                .bind(album.weight_kg)
                .fetch_one(&mut *tx)
                .await?
            } else {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO albums (series_id, title, tome, cover_url, ean, slug,
                                        summary, publisher, number_of_pages, publication_date,
                                        height_cm, width_cm, length_cm, weight_kg)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     RETURNING id",
                )
                .bind(series_id)
                .bind(&album.title)
                .bind(album.tome)
                .bind(&album.cover_url)
                .bind(&album.ean)
                .bind(&album.slug)
                .bind(&album.summary)
                .bind(&album.publisher)
                .bind(album.number_of_pages)
                .bind(&album.publication_date)
                .bind(album.height_cm)
                .bind(album.width_cm)
                .bind(album.length_cm)
                .bind(album.weight_kg)
                .fetch_one(&mut *tx)
                .await?
            };

            // Link album authors (authors must already exist from standalone files)
            for author in &album.authors {
                let author_db_id =
                    sqlx::query_scalar::<_, i64>("SELECT id FROM authors WHERE slug = ?")
                        .bind(&author.slug)
                        .fetch_one(&mut *tx)
                        .await?;

                sqlx::query(
                    "INSERT OR IGNORE INTO album_authors (album_id, author_id, role) VALUES (?, ?, ?)",
                )
                .bind(album_id)
                .bind(author_db_id)
                .bind(&author.role)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        println!(
            "Imported: {} ({} albums)",
            series.title,
            series.albums.len()
        );
    }

    if delete {
        // Delete series (cascades to albums → album_authors)
        let existing_series: Vec<String> =
            sqlx::query_scalar("SELECT bubble_id FROM series WHERE bubble_id IS NOT NULL")
                .fetch_all(pool)
                .await?;
        for bid in &existing_series {
            if !imported_series_bids.contains(bid) {
                sqlx::query("DELETE FROM series WHERE bubble_id = ?")
                    .bind(bid)
                    .execute(pool)
                    .await?;
                println!("Deleted series: {bid}");
            }
        }

        // Delete albums not in any imported series (orphaned from removed TOML entries)
        let existing_albums: Vec<String> =
            sqlx::query_scalar("SELECT bubble_id FROM albums WHERE bubble_id IS NOT NULL")
                .fetch_all(pool)
                .await?;
        for bid in &existing_albums {
            if !imported_album_bids.contains(bid) {
                sqlx::query("DELETE FROM albums WHERE bubble_id = ?")
                    .bind(bid)
                    .execute(pool)
                    .await?;
                println!("Deleted album: {bid}");
            }
        }

        // Delete authors not referenced anywhere in imported data
        let existing_authors: Vec<String> =
            sqlx::query_scalar("SELECT slug FROM authors WHERE slug IS NOT NULL")
                .fetch_all(pool)
                .await?;
        for slug in &existing_authors {
            if !imported_author_slugs.contains(slug) {
                sqlx::query("DELETE FROM authors WHERE slug = ?")
                    .bind(slug)
                    .execute(pool)
                    .await?;
                println!("Deleted author: {slug}");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;
    use tempfile::TempDir;

    async fn setup_db(dir: &Path) -> SqlitePool {
        let db_path = dir.join("test.db");
        let url = format!("sqlite:{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    async fn insert_test_data(pool: &SqlitePool) {
        sqlx::query(
            "INSERT INTO series (id, title, work_type, bubble_id, slug, year, number_of_albums)
             VALUES (1, 'Test Series', 'bd', 'bubble-series-1', 'test-series', 2020, 2)",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO albums (id, series_id, title, tome, bubble_id, slug, ean, summary, publisher, number_of_pages, publication_date)
             VALUES (1, 1, 'Album One', 1, 'bubble-album-1', 'album-one', '1234567890123', 'Summary one', 'Publisher A', 48, '2020-01-15')",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO albums (id, series_id, title, tome, bubble_id, slug, ean, summary, publisher, number_of_pages, publication_date)
             VALUES (2, 1, 'Album Two', 2, 'bubble-album-2', 'album-two', '1234567890124', 'Summary two', 'Publisher B', 52, '2021-06-20')",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO authors (id, display_name, bubble_id, slug, date_birth, date_death)
             VALUES (1, 'Author One', 'bubble-author-1', 'author-one', '1970-01-01', NULL)",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO authors (id, display_name, bubble_id, slug, bio, date_birth, date_death)
             VALUES (2, 'Author Two', 'bubble-author-2', 'author-two', 'A great author', '1980-05-15', '2023-12-01')",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO album_authors (album_id, author_id, role) VALUES (1, 1, 'Scénario')",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO album_authors (album_id, author_id, role) VALUES (1, 2, 'Dessin')",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO album_authors (album_id, author_id, role) VALUES (2, 1, 'Scénario')",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_export_import_round_trip() {
        let tmp = TempDir::new().unwrap();

        // Set up source DB with test data
        let db_dir = tmp.path().join("db1");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        // Export
        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Verify exported files exist
        assert!(export_dir.join("test-series.toml").exists());
        let authors_dir = tmp.path().join("export").join("authors");
        assert!(authors_dir.join("author-one.toml").exists());
        assert!(authors_dir.join("author-two.toml").exists());

        // Parse exported series and verify content
        let content = fs::read_to_string(export_dir.join("test-series.toml")).unwrap();
        let series: SeriesToml = toml::from_str(&content).unwrap();
        assert_eq!(series.title, "Test Series");
        assert_eq!(series.work_type, "bd");
        assert_eq!(series.year, Some(2020));
        assert_eq!(series.albums.len(), 2);
        assert_eq!(series.albums[0].title.as_deref(), Some("Album One"));
        assert_eq!(series.albums[0].tome, Some(1));
        assert_eq!(series.albums[0].authors.len(), 2);
        assert!(
            series.albums[0]
                .authors
                .iter()
                .any(|a| a.slug == "author-one" && a.role.as_deref() == Some("Scénario"))
        );
        assert!(
            series.albums[0]
                .authors
                .iter()
                .any(|a| a.slug == "author-two" && a.role.as_deref() == Some("Dessin"))
        );
        assert_eq!(series.albums[1].title.as_deref(), Some("Album Two"));
        assert_eq!(series.albums[1].authors.len(), 1);
        assert_eq!(series.albums[1].authors[0].slug, "author-one");
        assert_eq!(
            series.albums[1].authors[0].role.as_deref(),
            Some("Scénario")
        );

        // Import into a fresh DB
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();

        // Verify row counts
        let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(series_count, 1);

        let album_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(album_count, 2);

        let author_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(author_count, 2);

        let link_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM album_authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(link_count, 3);

        // Verify specific values survived the round trip
        let (title, work_type): (String, String) = sqlx::query_as(
            "SELECT title, work_type FROM series WHERE bubble_id = 'bubble-series-1'",
        )
        .fetch_one(&pool2)
        .await
        .unwrap();
        assert_eq!(title, "Test Series");
        assert_eq!(work_type, "bd");

        let (a_title, tome, ean): (Option<String>, Option<i64>, Option<String>) = sqlx::query_as(
            "SELECT title, tome, ean FROM albums WHERE bubble_id = 'bubble-album-1'",
        )
        .fetch_one(&pool2)
        .await
        .unwrap();
        assert_eq!(a_title.as_deref(), Some("Album One"));
        assert_eq!(tome, Some(1));
        assert_eq!(ean.as_deref(), Some("1234567890123"));

        // Verify roles survived the round trip
        let role: Option<String> = sqlx::query_scalar(
            "SELECT aa.role FROM album_authors aa
             JOIN authors au ON au.id = aa.author_id
             JOIN albums al ON al.id = aa.album_id
             WHERE au.slug = 'author-one' AND al.slug = 'album-one'",
        )
        .fetch_one(&pool2)
        .await
        .unwrap();
        assert_eq!(role.as_deref(), Some("Scénario"));
    }

    #[tokio::test]
    async fn test_import_idempotency() {
        let tmp = TempDir::new().unwrap();

        // Set up and export
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Import into fresh DB twice
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();
        import_data(&pool2, &export_dir, false).await.unwrap();

        // Counts should be identical to a single import
        let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(series_count, 1);

        let album_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(album_count, 2);

        let author_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(author_count, 2);

        let link_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM album_authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(link_count, 3);
    }

    #[tokio::test]
    async fn test_import_updates_existing() {
        let tmp = TempDir::new().unwrap();

        // Set up and export
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Import into fresh DB
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();

        // Modify the exported TOML and re-import
        let toml_path = export_dir.join("test-series.toml");
        let content = fs::read_to_string(&toml_path).unwrap();
        let mut series: SeriesToml = toml::from_str(&content).unwrap();
        series.title = "Updated Title".to_string();
        series.description = Some("New description".to_string());
        series.albums[0].summary = Some("Updated summary".to_string());
        fs::write(&toml_path, toml::to_string_pretty(&series).unwrap()).unwrap();

        import_data(&pool2, &export_dir, false).await.unwrap();

        // Verify updates were applied
        let (title, desc): (String, Option<String>) = sqlx::query_as(
            "SELECT title, description FROM series WHERE bubble_id = 'bubble-series-1'",
        )
        .fetch_one(&pool2)
        .await
        .unwrap();
        assert_eq!(title, "Updated Title");
        assert_eq!(desc.as_deref(), Some("New description"));

        let summary: Option<String> =
            sqlx::query_scalar("SELECT summary FROM albums WHERE bubble_id = 'bubble-album-1'")
                .fetch_one(&pool2)
                .await
                .unwrap();
        assert_eq!(summary.as_deref(), Some("Updated summary"));

        // Row counts unchanged
        let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(series_count, 1);

        let album_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(album_count, 2);
    }

    #[tokio::test]
    async fn test_export_filename_fallbacks() {
        let tmp = TempDir::new().unwrap();
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;

        // Series with slug
        sqlx::query(
            "INSERT INTO series (id, title, work_type, slug) VALUES (1, 'Has Slug', 'bd', 'has-slug')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Series with bubble_id but no slug
        sqlx::query(
            "INSERT INTO series (id, title, work_type, bubble_id) VALUES (2, 'Has Bubble', 'bd', 'bid-123')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Series with neither
        sqlx::query("INSERT INTO series (id, title, work_type) VALUES (3, 'Has Nothing', 'bd')")
            .execute(&pool)
            .await
            .unwrap();

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        assert!(export_dir.join("has-slug.toml").exists());
        assert!(export_dir.join("bubble-bid-123.toml").exists());
        assert!(export_dir.join("series-3.toml").exists());
    }

    #[tokio::test]
    async fn test_author_dates_round_trip() {
        let tmp = TempDir::new().unwrap();
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Check author TOML files contain dates and bio
        let authors_dir = tmp.path().join("export").join("authors");
        let content = fs::read_to_string(authors_dir.join("author-two.toml")).unwrap();
        let author: AuthorToml = toml::from_str(&content).unwrap();
        assert_eq!(author.date_birth.as_deref(), Some("1980-05-15"));
        assert_eq!(author.date_death.as_deref(), Some("2023-12-01"));
        assert_eq!(author.bio.as_deref(), Some("A great author"));

        // Author One has birth but no death
        let content = fs::read_to_string(authors_dir.join("author-one.toml")).unwrap();
        let author: AuthorToml = toml::from_str(&content).unwrap();
        assert_eq!(author.date_birth.as_deref(), Some("1970-01-01"));
        assert!(author.date_death.is_none());

        // Import into fresh DB and verify dates survive
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();

        let (date_birth, date_death, bio): (Option<String>, Option<String>, Option<String>) =
            sqlx::query_as(
                "SELECT date_birth, date_death, bio FROM authors WHERE bubble_id = 'bubble-author-2'",
            )
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(date_birth.as_deref(), Some("1980-05-15"));
        assert_eq!(date_death.as_deref(), Some("2023-12-01"));
        assert_eq!(bio.as_deref(), Some("A great author"));
    }

    #[tokio::test]
    async fn test_delete_removes_missing_entries() {
        let tmp = TempDir::new().unwrap();

        // Set up source DB with test data and export
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Import into a fresh DB
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authors")
                .fetch_one(&pool2)
                .await
                .unwrap(),
            2
        );

        // Delete author TOML file AND remove from album authors in series TOML
        let authors_dir = tmp.path().join("export").join("authors");
        fs::remove_file(authors_dir.join("author-two.toml")).unwrap();

        let toml_path = export_dir.join("test-series.toml");
        let content = fs::read_to_string(&toml_path).unwrap();
        let mut series: SeriesToml = toml::from_str(&content).unwrap();
        for album in &mut series.albums {
            album.authors.retain(|a| a.slug != "author-two");
        }
        fs::write(&toml_path, toml::to_string_pretty(&series).unwrap()).unwrap();

        // Without --delete: author stays
        import_data(&pool2, &export_dir, false).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authors")
                .fetch_one(&pool2)
                .await
                .unwrap(),
            2
        );

        // With --delete: author is removed
        import_data(&pool2, &export_dir, true).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authors")
                .fetch_one(&pool2)
                .await
                .unwrap(),
            1
        );

        let name: String = sqlx::query_scalar("SELECT display_name FROM authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(name, "Author One");
    }

    #[tokio::test]
    async fn test_delete_cascades_series_removal() {
        let tmp = TempDir::new().unwrap();

        // Set up and export
        let db_dir = tmp.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        let pool = setup_db(&db_dir).await;
        insert_test_data(&pool).await;

        let export_dir = tmp.path().join("export").join("series");
        export_data(&pool, &export_dir).await.unwrap();

        // Import into fresh DB
        let db_dir2 = tmp.path().join("db2");
        fs::create_dir_all(&db_dir2).unwrap();
        let pool2 = setup_db(&db_dir2).await;
        import_data(&pool2, &export_dir, false).await.unwrap();

        // Delete the series TOML file
        fs::remove_file(export_dir.join("test-series.toml")).unwrap();

        // Re-import with --delete
        import_data(&pool2, &export_dir, true).await.unwrap();

        // Series and its albums should be gone (CASCADE)
        let series_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM series")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(series_count, 0);

        let album_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(album_count, 0);

        let link_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM album_authors")
            .fetch_one(&pool2)
            .await
            .unwrap();
        assert_eq!(link_count, 0);
    }
}
