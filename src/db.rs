use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::OnceLock;

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Get a clone of the global database pool.
pub fn pool() -> SqlitePool {
    POOL.get().expect("Database pool not initialized").clone()
}

pub async fn create_pool() -> SqlitePool {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:mybd.db".to_string());

    let options = SqliteConnectOptions::from_str(&database_url)
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create database pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    backfill_avatars(&pool).await;
    crate::server::slug::backfill_slugs(&pool).await;

    POOL.set(pool.clone()).expect("Pool already initialized");
    pool
}

/// One-time backfill: set avatar_url for any user that still has NULL.
async fn backfill_avatars(pool: &SqlitePool) {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, email FROM users WHERE avatar_url IS NULL")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    for (id, email) in rows {
        let url = crate::gravatar::gravatar_url(&email, 200);
        sqlx::query("UPDATE users SET avatar_url = ? WHERE id = ?")
            .bind(&url)
            .bind(id)
            .execute(pool)
            .await
            .ok();
    }
}
